//! The temp module end to end: a fixture sandbox whose two temporary places lie beside its home (outside the scan
//! root), the real scanner (its reader seen through the facts lens), the `temp` module, the real settings file, scan
//! cache and journal, and the real `DiskCleaner` reaching only inside the sandbox, with a fake Trash. The nine flows
//! of the design's section 9, "Core e2e"; each is a function that asserts and gives its report, and the reports of
//! the flows whose times are the fixture's are the golden file (flows 2 and 3 make their entries now, on the real
//! disk).
//!
//! `UPDATE_GOLDEN=1 cargo nextest run -p fresher-e2e --test temp` rewrites `fixtures/golden/temp-flow.json`; the diff
//! is reviewed.

#[cfg(test)]
mod support;

#[cfg(test)]
#[path = "temp_home/mod.rs"]
mod flow;

#[cfg(test)]
mod flows {
    use std::fs::{self, File, FileTimes};
    use std::os::unix::ffi::OsStrExt as _;
    use std::os::unix::net::UnixListener;
    use std::path::{Path, PathBuf};
    use std::sync::mpsc;
    use std::thread;
    use std::time::{Duration, SystemTime};

    use fresher_core::{
        CommandError, DangerLevel, Event, HiddenBy, IgnoreTarget, IgnoredEntry, ItemOutcome, ModuleGroup, PartKind,
        PutBackOutcome, ScanKind, SettingUpdate, SettingValue, SkipCode, SkipReason,
    };
    use fresher_testing::{FIXTURE_NOW, TreeReport, make_pipe};
    use serde_json::{Value, json};

    use crate::flow::{
        CATEGORY, DAY, Flow, Home, SHARED_TEMP, Setup, USER_TEMP, contents, left_out, newest_change, select_all,
    };

    /// The three old items of the flow home, relative to the sandbox, sorted.
    const OLD: [&str; 3] = [
        "private/tmp/old-build.log",
        "private/var/folders/xx/sandbox/T/node-compile-cache",
        "private/var/folders/xx/sandbox/T/tmp.IKKNWYIRnq",
    ];
    /// A socket's path, its last byte a NUL, fits in 104 bytes.
    const SOCKET_PATH_MAX: usize = 103;
    /// How long a flow on real sockets and pipes may take before it counts as blocked.
    const NEVER_BLOCKS: Duration = Duration::from_secs(60);

    fn offered(paths: &[&str]) -> Vec<(String, String)> {
        paths.iter().map(|path| (CATEGORY.to_owned(), (*path).to_owned())).collect()
    }

    fn reasons(pairs: &[(&str, u32)]) -> Vec<(String, u32)> {
        pairs.iter().map(|(reason, items)| (format!("temp.left-out.{reason}"), *items)).collect()
    }

    fn tree(flow: &Flow, relative: &str) -> Value {
        serde_json::to_value(TreeReport::of(&flow.path(relative)).unwrap()).unwrap()
    }

    fn hidden(flow: &Flow) -> Vec<(String, HiddenBy)> {
        let overview = flow.engine.overview().unwrap();
        overview.hidden_modules.iter().map(|one| (one.module.as_str().to_owned(), one.by)).collect()
    }

    fn days_setting(days: i64) -> SettingUpdate {
        SettingUpdate { key: "temp.older-than-days".into(), value: SettingValue::Number(days) }
    }

    /// Sets the modification and access times of `path` to `at` seconds.
    fn set_times(path: &Path, at: u64) {
        let at = SystemTime::UNIX_EPOCH + Duration::from_secs(at);
        File::open(path).unwrap().set_times(FileTimes::new().set_modified(at).set_accessed(at)).unwrap();
    }

    /// Runs `flow` on a thread of its own; fails when it has not ended within `NEVER_BLOCKS`.
    fn within_timeout(flow: fn() -> Value) -> Value {
        let (sender, receiver) = mpsc::channel();
        thread::spawn(move || sender.send(flow()));
        receiver.recv_timeout(NEVER_BLOCKS).expect("the flow blocked or failed")
    }

    /// Flow 1: scan, review, confirm to the Trash, put back.
    pub(crate) fn clean_and_put_back() -> Value {
        let flow = Flow::new(Home::Flow);
        let events = flow.scan();

        assert_eq!(flow.findings(), offered(&OLD));
        assert_eq!(left_out(&events), reasons(&[("recent", 1), ("system", 1)]));
        let module = flow.engine.modules().into_iter().find(|module| module.id.as_str() == "temp").unwrap();
        assert_eq!(module.group, ModuleGroup::System);
        let before: Vec<_> = OLD.iter().map(|old| contents(&flow.path(old))).collect();
        assert!(before.iter().all(|files| files.values().all(|bytes| !bytes.is_empty())));

        let view = flow.engine.prepare_cleanup(&select_all()).unwrap();

        assert_eq!(view.summary.count, 3);
        assert_eq!(view.summary.highest, DangerLevel::Safe);
        assert_eq!(flow.planned(&view), OLD);
        let (cleanup, cleaned) = flow.clean(&view);
        let outcomes = flow.outcomes(&view, &cleaned);
        assert!(outcomes.iter().all(|(_, outcome)| *outcome == ItemOutcome::Moved), "{outcomes:?}");
        assert!(OLD.iter().all(|old| !flow.path(old).exists()));
        assert!(flow.path(&format!("{USER_TEMP}/fresh.tmp")).is_file(), "the fresh file stays");
        assert!(flow.path(&format!("{USER_TEMP}/updater.pkg")).is_file(), "the system's package stays");
        assert_eq!(flow.findings(), []);
        assert_eq!(fs::read_dir(flow.sandbox.trash()).unwrap().count(), 3);
        let journal = flow.journal();
        assert_eq!(journal.iter().filter(|record| record["type"] == "item").count(), 3);
        let tree_after_cleanup = json!({ "places": tree(&flow, "private"), "home": tree(&flow, "home") });

        let back = flow.engine.put_back(cleanup, None).unwrap();

        assert!(back.items.iter().all(|item| item.outcome == PutBackOutcome::PutBack), "{back:?}");
        let after: Vec<_> = OLD.iter().map(|old| contents(&flow.path(old))).collect();
        assert_eq!(after, before, "put back byte for byte");
        let report = json!({
            "findings": offered(&OLD),
            "leftOut": left_out(&events),
            "plan": view,
            "outcomes": outcomes,
            "treeAfterCleanup": tree_after_cleanup,
            "putBack": back,
            "journal": flow.journal(),
            "treeAfterPutBack": { "places": tree(&flow, "private"), "home": tree(&flow, "home") },
        });
        flow.relative(&report)
    }

    /// The folder `T/unpacked` with two files, every time set to 2019 (so its status changes are now), and its
    /// entries.
    fn unpacked(flow: &Flow) -> Vec<PathBuf> {
        const IN_2019: u64 = 1_559_347_200;
        let folder = flow.path(&format!("{USER_TEMP}/unpacked"));
        fs::create_dir_all(folder.join("lib")).unwrap();
        fs::write(folder.join("README"), b"read me").unwrap();
        fs::write(folder.join("lib/data.bin"), [7_u8; 4096]).unwrap();
        let entries = [folder.join("README"), folder.join("lib/data.bin"), folder.join("lib"), folder.clone()];
        for entry in &entries {
            set_times(entry, IN_2019);
        }
        entries.into()
    }

    /// Flow 2, first half: an unpacked folder with 2019 modification times, set on the real disk, is kept, its status
    /// change a day ago; nothing is stated.
    pub(crate) fn real_times_kept() -> (Flow, u64, Value) {
        let mut flow = Flow::new(Home::Bare);
        let changed = newest_change(&unpacked(&flow));
        flow.restart(Setup::at(changed + DAY));

        let events = flow.scan();

        assert!(flow.stated.is_empty());
        assert_eq!(flow.findings(), []);
        assert_eq!(left_out(&events), reasons(&[("recent", 1)]));
        let report = json!({ "findings": flow.findings(), "leftOut": left_out(&events) });
        (flow, changed, report)
    }

    /// Flow 2, second half: eight days after its status change, the same folder is offered.
    pub(crate) fn real_times_later(mut flow: Flow, changed: u64) -> Value {
        flow.restart(Setup::at(changed + 8 * DAY));

        let events = flow.scan();

        assert_eq!(flow.findings(), offered(&["private/var/folders/xx/sandbox/T/unpacked"]));
        assert_eq!(left_out(&events), []);
        json!({ "findings": flow.findings(), "leftOut": left_out(&events) })
    }

    /// Flow 3: a real pipe and a real socket in the shared place, eight days after they were made, are left out as
    /// special while a plain file made with them is offered; the scan never opens the pipe. The plain file replaced by
    /// a pipe after the plan is skipped: the guard never blocks on it.
    pub(crate) fn real_sockets_and_pipes() -> Value {
        let mut flow = Flow::new(Home::Bare);
        let shared = flow.path(SHARED_TEMP);
        let socket = shared.join("s.sock");
        assert!(
            socket.as_os_str().as_bytes().len() <= SOCKET_PATH_MAX,
            "the socket's path {} is too long to bind",
            socket.display()
        );
        let _listener = UnixListener::bind(&socket).unwrap();
        make_pipe(&shared.join("fifo")).unwrap();
        fs::write(shared.join("plain.log"), b"plain").unwrap();
        let entries = ["s.sock", "fifo", "plain.log"].map(|name| shared.join(name));
        flow.restart(Setup::at(newest_change(&entries) + 8 * DAY));

        let events = flow.scan();

        assert!(flow.stated.is_empty());
        assert_eq!(flow.findings(), offered(&["private/tmp/plain.log"]));
        assert_eq!(left_out(&events), reasons(&[("special", 2)]));
        let view = flow.engine.prepare_cleanup(&select_all()).unwrap();
        make_pipe(&shared.join("plain.next")).unwrap();
        fs::rename(shared.join("plain.next"), shared.join("plain.log")).unwrap();
        let (_, cleaned) = flow.clean(&view);
        let outcomes = flow.outcomes(&view, &cleaned);
        assert_eq!(
            outcomes,
            [("private/tmp/plain.log".to_owned(), ItemOutcome::Skipped { reason: SkipReason::KindChanged })]
        );
        assert!(entries.iter().all(|entry| entry.symlink_metadata().is_ok()), "nothing went");
        json!({ "findings": flow.findings(), "leftOut": left_out(&events), "outcomes": outcomes })
    }

    /// Flow 3 on a thread of its own, within its timeout.
    pub(crate) fn real_sockets_and_pipes_within_timeout() -> Value {
        within_timeout(real_sockets_and_pipes)
    }

    /// Flow 4: an offered file rewritten after the plan was shown (a new file put in its place, as a program saves) is
    /// skipped; the rest goes.
    pub(crate) fn changed_before_the_act() -> Value {
        let flow = Flow::new(Home::Flow);
        flow.scan();
        let view = flow.engine.prepare_cleanup(&select_all()).unwrap();
        let rewritten = flow.path(&format!("{USER_TEMP}/tmp.IKKNWYIRnq"));
        let fresh = flow.path(&format!("{USER_TEMP}/tmp.IKKNWYIRnq.new"));
        fs::write(&fresh, b"written again").unwrap();
        fs::rename(&fresh, &rewritten).unwrap();

        let (_, cleaned) = flow.clean(&view);

        let outcomes = flow.outcomes(&view, &cleaned);
        assert_eq!(
            outcomes,
            [
                (OLD[0].to_owned(), ItemOutcome::Moved),
                (OLD[1].to_owned(), ItemOutcome::Moved),
                (OLD[2].to_owned(), ItemOutcome::Skipped { reason: SkipReason::Replaced }),
            ]
        );
        assert_eq!(fs::read(&rewritten).unwrap(), b"written again");
        assert!(!flow.path(OLD[0]).exists() && !flow.path(OLD[1]).exists());
        json!({ "outcomes": outcomes })
    }

    /// Flow 5: a restore at launch reads no folder, so it knows no item's times: it emits nothing inside a place
    /// (design, decision 21), so nothing is offered or left out there, while the map still shows the places; a rescan
    /// offers again.
    pub(crate) fn restored_at_launch() -> Value {
        let mut flow = Flow::new(Home::Flow);
        flow.scan();
        flow.restart(Setup::at(FIXTURE_NOW));

        let restored = flow.restore();

        assert!(matches!(restored.last(), Some(Event::ScanFinished { kind: ScanKind::Restored { .. }, .. })));
        assert_eq!(flow.findings(), []);
        assert_eq!(left_out(&restored), []);
        let map = flow.engine.storage_map().unwrap();
        assert_eq!(map.volumes[0].places().len(), 2);
        let explained = map.volumes[0].parts().iter().find(|part| part.kind == PartKind::Explained);
        assert!(explained.is_none_or(|part| part.sizes.logical.bytes() == 0), "{explained:?}");
        let first = json!({ "findings": flow.findings(), "leftOut": left_out(&restored) });
        let events = flow.scan();
        assert_eq!(flow.findings(), offered(&OLD));
        assert_eq!(left_out(&events), reasons(&[("recent", 1), ("system", 1)]));
        json!({ "restored": first, "rescan": { "findings": flow.findings(), "leftOut": left_out(&events) } })
    }

    /// Flow 6, first half: fewer days than seven is refused; the value stays seven.
    pub(crate) fn fewer_days_refused() -> Value {
        let flow = Flow::new(Home::Flow);

        let refused = flow.engine.update_setting(days_setting(3));

        assert!(matches!(refused, Err(CommandError::InvalidSettingValue { .. })), "{refused:?}");
        let settings = flow.engine.settings().unwrap();
        let value = settings.values.iter().find(|(key, _)| key.as_str() == "temp.older-than-days").map(|(_, v)| v);
        assert_eq!(value, Some(&SettingValue::Number(7)));
        json!({ "refused": refused.unwrap_err(), "value": value })
    }

    /// Flow 6, second half: fourteen days keeps the item ten days old as recent.
    pub(crate) fn more_days_keep_more() -> Value {
        let flow = Flow::new(Home::Flow);
        flow.engine.update_setting(days_setting(14)).unwrap();

        let events = flow.scan();

        assert_eq!(flow.findings(), offered(&OLD[1..]));
        assert_eq!(left_out(&events), reasons(&[("recent", 2), ("system", 1)]));
        json!({ "findings": flow.findings(), "leftOut": left_out(&events) })
    }

    /// Flow 7: ignoring the module, then the group System, hides every answer; restoring brings it back.
    pub(crate) fn ignoring() -> Value {
        let flow = Flow::new(Home::Flow);
        flow.scan();
        assert_eq!(flow.findings(), offered(&OLD));

        flow.engine.ignore(&IgnoreTarget::Module { module: "temp".into() }).unwrap();

        assert_eq!(flow.findings(), []);
        assert_eq!(flow.engine.overview().unwrap().totals.len(), 0);
        assert_eq!(hidden(&flow), [("temp".to_owned(), HiddenBy::Module)]);
        flow.engine.ignore(&IgnoreTarget::Group { group: ModuleGroup::System }).unwrap();
        assert_eq!(hidden(&flow), [("temp".to_owned(), HiddenBy::Group)]);
        assert_eq!(flow.engine.prepare_cleanup(&select_all()).unwrap_err(), CommandError::NothingToClean);

        flow.engine.restore_ignored(&IgnoredEntry::Group { group: ModuleGroup::System }).unwrap();
        assert_eq!(hidden(&flow), [("temp".to_owned(), HiddenBy::Module)]);
        flow.engine.restore_ignored(&IgnoredEntry::Module { module: "temp".into() }).unwrap();

        assert_eq!(flow.findings(), offered(&OLD));
        assert_eq!(hidden(&flow), []);
        json!({ "findings": flow.findings() })
    }

    /// Flow 8: the two places are by-place roots after the home, by their names; the offered items are explained.
    pub(crate) fn storage_map() -> Value {
        let flow = Flow::new(Home::Flow);
        flow.scan();

        let map = flow.engine.storage_map().unwrap();

        let volume = &map.volumes[0];
        assert_eq!(volume.roots().len(), 3);
        let mut names: Vec<&str> = volume.places().iter().map(|place| place.name.key().as_str()).collect();
        names.sort_unstable();
        assert_eq!(names, ["core.place.shared-temp.name", "core.place.user-temp.name"]);
        assert!(volume.places().iter().all(|place| volume.roots()[1..].contains(&place.root)));
        let explained = volume.parts().iter().find(|part| part.kind == PartKind::Explained).unwrap();
        assert_eq!(explained.sizes.logical.bytes(), 65_536 + 16_384 + 32_768 + 16_384);
        let total: u64 = volume.parts().iter().map(|part| part.sizes.allocated.bytes()).sum();
        assert_eq!(total, volume.capacity().bytes());
        json!({ "places": names, "explained": explained.sizes.logical.bytes() })
    }

    /// Flow 9, first half: the cleaner's roots are the places the last scan walked. With no module naming the places
    /// the scan walks none, and the items of the cleanup are refused their way back as outside every root.
    pub(crate) fn place_no_longer_walked() -> (Flow, fresher_core::CleanupId, Value) {
        let mut flow = Flow::new(Home::Flow);
        flow.scan();
        let view = flow.engine.prepare_cleanup(&select_all()).unwrap();
        let (cleanup, _) = flow.clean(&view);
        flow.restart(Setup::without_temp());
        flow.scan();

        let refused = flow.engine.put_back(cleanup, None).unwrap();

        let outside = PutBackOutcome::Refused { reason: SkipCode::OutsideRoots };
        assert_eq!(refused.items.len(), 3);
        assert!(refused.items.iter().all(|item| item.outcome == outside), "{refused:?}");
        assert!(OLD.iter().all(|old| !flow.path(old).exists()));
        assert_eq!(fs::read_dir(flow.sandbox.trash()).unwrap().count(), 3);
        let report = flow.relative(&json!({ "putBack": refused }));
        (flow, cleanup, report)
    }

    /// Flow 9, second half: once the module walks the places again, the same items go back.
    pub(crate) fn place_walked_again(mut flow: Flow, cleanup: fresher_core::CleanupId) -> Value {
        flow.restart(Setup::at(FIXTURE_NOW));
        flow.scan();

        let back = flow.engine.put_back(cleanup, None).unwrap();

        assert!(back.items.iter().all(|item| item.outcome == PutBackOutcome::PutBack), "{back:?}");
        assert!(OLD.iter().all(|old| flow.path(old).exists()));
        flow.relative(&json!({ "putBack": back }))
    }
}

mod negative {
    use crate::flows::{
        changed_before_the_act, fewer_days_refused, ignoring, place_no_longer_walked,
        real_sockets_and_pipes_within_timeout, real_times_kept, restored_at_launch,
    };

    #[test]
    fn an_unpacked_folder_with_old_modification_times_is_kept_by_its_status_change() {
        real_times_kept();
    }

    #[test]
    fn a_real_pipe_and_socket_are_left_out_and_nothing_blocks() {
        real_sockets_and_pipes_within_timeout();
    }

    #[test]
    fn a_file_rewritten_after_the_plan_is_skipped_and_the_rest_goes() {
        changed_before_the_act();
    }

    #[test]
    fn a_restore_at_launch_offers_nothing_and_a_rescan_offers_again() {
        restored_at_launch();
    }

    #[test]
    fn fewer_than_seven_days_is_refused_and_the_value_stays_seven() {
        fewer_days_refused();
    }

    #[test]
    fn ignoring_the_module_and_the_group_hides_everything_and_restoring_brings_it_back() {
        ignoring();
    }

    #[test]
    fn an_item_of_a_place_no_longer_walked_is_refused_its_way_back() {
        place_no_longer_walked();
    }
}

mod positive {
    use crate::flows::{
        changed_before_the_act, clean_and_put_back, fewer_days_refused, ignoring, more_days_keep_more,
        place_no_longer_walked, place_walked_again, real_sockets_and_pipes_within_timeout, real_times_kept,
        real_times_later, restored_at_launch, storage_map,
    };
    use crate::support::check_golden;

    #[test]
    fn the_old_items_go_to_the_trash_and_come_back_byte_for_byte() {
        clean_and_put_back();
    }

    #[test]
    fn the_unpacked_folder_is_offered_eight_days_after_its_status_change() {
        let (flow, changed, _) = real_times_kept();

        real_times_later(flow, changed);
    }

    #[test]
    fn fourteen_days_keep_the_item_ten_days_old() {
        more_days_keep_more();
    }

    #[test]
    fn the_places_are_by_place_roots_and_the_offered_items_are_explained() {
        storage_map();
    }

    #[test]
    fn the_items_go_back_once_the_places_are_walked_again() {
        let (flow, cleanup, _) = place_no_longer_walked();

        place_walked_again(flow, cleanup);
    }

    #[test]
    fn the_reports_of_the_flows_match_the_golden_file() {
        let (no_longer, cleanup, refused) = place_no_longer_walked();
        let (kept, changed, real_times) = real_times_kept();
        let report = serde_json::json!({
            "cleanAndPutBack": clean_and_put_back(),
            "realTimesKept": real_times,
            "realTimesLater": real_times_later(kept, changed),
            "realSocketsAndPipes": real_sockets_and_pipes_within_timeout(),
            "changedBeforeTheAct": changed_before_the_act(),
            "restoredAtLaunch": restored_at_launch(),
            "fewerDaysRefused": fewer_days_refused(),
            "moreDaysKeepMore": more_days_keep_more(),
            "ignoring": ignoring(),
            "storageMap": storage_map(),
            "placeNoLongerWalked": refused,
            "placeWalkedAgain": place_walked_again(no_longer, cleanup),
        });

        check_golden("temp-flow", &report);
    }
}
