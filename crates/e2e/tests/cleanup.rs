//! A cleanup end to end, on files: the engine scans `fixtures/homes/cleanup.toml` with the reference walk, the
//! `trial` module and a build module of the test's own; the selection becomes a plan, the confirmation a cleanup
//! through the real `DiskCleaner` (`Reach::Sandbox`, a fake Trash next to the home) and the real journal; then the
//! history and put back. Outside the fixture tree nothing is touched, and inside it the link's target outside the
//! scan root never changes.
//!
//! `UPDATE_GOLDEN=1 cargo nextest run -p fresher-e2e --test cleanup` rewrites `fixtures/golden/cleanup-*.json`; the
//! diff is reviewed.

#[cfg(test)]
mod support;

#[cfg(test)]
#[path = "cleaning/mod.rs"]
#[expect(dead_code, reason = "the cleanup home is shared; only the agreement flows restart it")]
mod cleaning;

#[cfg(test)]
mod checks {
    use fresher_core::CleanupItemView;
    use fresher_testing::TreeReport;

    use crate::cleaning::Home;

    /// The link's target outside the scan root and the hard link's other name, as the fixture made them.
    pub(crate) fn assert_outside_untouched(home: &Home, before: &TreeReport) {
        assert_eq!(&outside(home), before, "the outside target changed");
        assert_eq!(std::fs::metadata(home.path("home/notes/shared.bin")).unwrap().len(), 4096);
    }

    /// The index of the item at `path` (relative to the fixture) in `items`.
    pub(crate) fn index_of(home: &Home, items: &[CleanupItemView], path: &str) -> u32 {
        let path = home.path(path);
        items.iter().find(|item| item.path.as_deref() == Some(path.as_path())).unwrap().index
    }

    /// The folder outside the scan root that the link points into.
    pub(crate) fn outside(home: &Home) -> TreeReport {
        TreeReport::of(&home.path("outside")).unwrap()
    }

    /// Nothing was removed and nothing journalled: the tree as before, no cleanup in the history.
    pub(crate) fn assert_nothing_done(home: &Home, before: &TreeReport) {
        assert_eq!(&home.tree(), before, "the tree changed");
        assert!(!home.path("trash").exists(), "something went to the Trash");
        assert_eq!(home.engine.cleanups(0, 10).total, 0);
        assert!(home.journal().iter().all(|record| record.get("type").is_none()), "{:?}", home.journal());
    }
}

mod negative {
    use std::fs::{self, Permissions};
    use std::os::unix::fs::PermissionsExt as _;
    use std::time::Duration;

    use fresher_core::{BlockedReason, CommandError, DangerLevel, LeftOut, PLAN_LIFETIME, SkipReason};
    use serde_json::json;

    use crate::checks::assert_nothing_done;
    use crate::cleaning::{JournalKind, Setup, echo, home};

    #[test]
    fn a_confirmation_with_a_wrong_count_moves_and_journals_nothing() {
        let home = home(Setup::DEFAULT);
        home.scan();
        let before = home.tree();
        let view = home.prepare();
        let wrong = json!({ "plan": view.plan, "count": view.summary.count + 1,
                            "allocated": view.summary.sizes.allocated, "highest": view.summary.highest,
                            "method": "trash", "highRiskUnderstood": view.summary.highest == DangerLevel::High });

        let error = home.engine.confirm_cleanup(&serde_json::from_value(wrong).unwrap());

        assert_eq!(error, Err(CommandError::PlanMismatch));
        assert_nothing_done(&home, &before);
        assert_eq!(home.engine.confirm_cleanup(&echo(&view, "trash")), Err(CommandError::PlanGone));
    }

    #[test]
    fn a_plan_older_than_its_lifetime_is_refused() {
        let home = home(Setup::DEFAULT);
        home.scan();
        let before = home.tree();
        let view = home.prepare();
        home.clock.advance(PLAN_LIFETIME + Duration::from_secs(1));

        let error = home.engine.confirm_cleanup(&echo(&view, "trash"));

        assert_eq!(error, Err(CommandError::PlanGone));
        assert_nothing_done(&home, &before);
    }

    #[test]
    fn a_cleanup_outside_a_sandbox_is_refused_and_journals_nothing() {
        let home = home(Setup { marked: false, ..Setup::DEFAULT });
        home.scan();
        let before = home.tree();

        let view = home.prepare();

        let reasons: Vec<&SkipReason> = view.left_out.iter().map(LeftOut::reason).collect();
        let not_in_sandbox = &SkipReason::NotInSandbox;
        assert_eq!(view.summary.count, 0);
        assert_eq!(
            reasons,
            [
                not_in_sandbox,
                not_in_sandbox,
                not_in_sandbox,
                &SkipReason::Covered,
                not_in_sandbox,
                not_in_sandbox,
                not_in_sandbox
            ]
        );
        let error = home.engine.confirm_cleanup(&echo(&view, "trash"));

        assert_eq!(error, Err(CommandError::NothingToClean));
        assert_nothing_done(&home, &before);
        assert_eq!(home.engine.confirm_cleanup(&echo(&view, "trash")), Err(CommandError::PlanGone));
    }

    #[test]
    fn a_journal_that_cannot_be_written_blocks_the_cleanup() {
        let home = home(Setup::DEFAULT);
        home.scan();
        let before = home.tree();
        let view = home.prepare();
        fs::set_permissions(home.data.path(), Permissions::from_mode(0o500)).unwrap();

        let error = home.engine.confirm_cleanup(&echo(&view, "trash"));

        fs::set_permissions(home.data.path(), Permissions::from_mode(0o700)).unwrap();
        assert_eq!(error, Err(CommandError::CleaningBlocked { reason: BlockedReason::Journal }));
        assert_nothing_done(&home, &before);
    }

    #[test]
    fn a_full_disk_blocks_the_cleanup() {
        let home = home(Setup { journal: JournalKind::DiskFull, ..Setup::DEFAULT });
        home.scan();
        let before = home.tree();
        let view = home.prepare();

        let error = home.engine.confirm_cleanup(&echo(&view, "trash"));

        assert_eq!(error, Err(CommandError::CleaningBlocked { reason: BlockedReason::DiskFull }));
        assert_nothing_done(&home, &before);
    }
}

mod positive {
    use std::fs;
    use std::time::Duration;

    use fresher_core::{Ended, Event, PutBackOutcome, TrashState};
    use serde_json::json;

    use crate::checks::{assert_outside_untouched, index_of, outside};
    use crate::cleaning::{Setup, echo, home, until_finished};
    use crate::support::check_golden;

    #[test]
    fn the_whole_path_moves_journals_lists_and_puts_back() {
        let home = home(Setup::DEFAULT);
        home.scan();
        let outside_before = outside(&home);
        let view = home.prepare();

        let (cleanup, events) = home.clean(&view, "trash");

        let tree_after_cleanup = home.tree();
        let findings = home.findings();
        let history = home.engine.cleanups(0, 10);
        let detail = home.engine.cleanup(cleanup).unwrap();
        let cache = index_of(&home, &detail.items, "home/sandbox/Sample app cache");
        let back = home.engine.put_back(cleanup, Some(&[cache])).unwrap();
        let again = home.engine.put_back(cleanup, Some(&[cache])).unwrap();
        assert_eq!(back.items[0].outcome, PutBackOutcome::PutBack);
        assert_eq!(again.items[0].outcome, PutBackOutcome::PutBack);
        assert_eq!(detail.items[cache as usize].trash, Some(TrashState::InTrash));
        assert!(home.path("home/sandbox/Sample app cache/nested/blob.bin").exists());
        assert!(!home.path("home/sandbox/fresher-e2e-build").exists());
        assert!(home.path("home/sandbox/keep-me.txt").exists());
        assert!(home.path("home/sandbox/Old sample downloads").is_dir());
        assert_outside_untouched(&home, &outside_before);
        let report = json!({
            "plan": view,
            "events": events,
            "findingsAfter": findings,
            "overviewAfter": home.engine.overview().unwrap(),
            "treeAfterCleanup": tree_after_cleanup,
            "history": history,
            "detail": detail,
            "putBack": back,
            "putBackAgain": again,
            "detailAfterPutBack": home.engine.cleanup(cleanup).unwrap(),
            "journal": home.journal(),
            "treeAfterPutBack": home.tree(),
        });
        check_golden("cleanup-whole-path", &home.relative(&report));
    }

    #[test]
    fn a_cancel_after_the_first_item_leaves_the_rest_unreached() {
        let mut home = home(Setup { pause: true, ..Setup::DEFAULT });
        home.scan();
        let outside_before = outside(&home);
        let view = home.prepare();
        let events = home.engine.subscribe();
        let paused = home.paused.take().unwrap();

        let cleanup = home.engine.confirm_cleanup(&echo(&view, "trash")).unwrap();
        paused.recv_timeout(Duration::from_secs(30)).unwrap();
        home.engine.cancel_cleanup().unwrap();

        let events = until_finished(&events);
        assert!(
            matches!(events.last(), Some(Event::CleanupFinished { report, .. }) if report.ended == Ended::Cancelled)
        );
        let detail = home.engine.cleanup(cleanup).unwrap();
        assert_eq!(detail.summary.ended, Ended::Cancelled);
        assert_outside_untouched(&home, &outside_before);
        let report = json!({ "events": events, "journal": home.journal(), "tree": home.tree() });
        check_golden("cleanup-cancelled", &home.relative(&report));
    }

    #[test]
    fn an_item_replaced_after_prepare_is_skipped_and_the_rest_done() {
        let home = home(Setup::DEFAULT);
        home.scan();
        let outside_before = outside(&home);
        let view = home.prepare();
        let cache = home.path("home/sandbox/Sample app cache");
        fs::rename(&cache, home.path("home/sandbox/Sample app cache-old")).unwrap();
        fs::create_dir(&cache).unwrap();
        fs::write(cache.join("blob.bin"), b"a new folder of the same name").unwrap();

        let (_, events) = home.clean(&view, "trash");

        assert!(cache.join("blob.bin").exists(), "the replaced folder stays");
        assert!(!home.path("home/sandbox/fresher-e2e-build").exists());
        assert_outside_untouched(&home, &outside_before);
        let report = json!({ "events": events, "journal": home.journal(), "tree": home.tree() });
        check_golden("cleanup-replaced", &home.relative(&report));
    }

    #[test]
    fn a_permanent_cleanup_deletes_and_leaves_the_link_target_alone() {
        let home = home(Setup::DEFAULT);
        home.scan();
        let outside_before = outside(&home);
        let view = home.prepare();
        assert!(view.permanent_available);

        let (cleanup, events) = home.clean(&view, "permanent");

        assert!(!home.path("trash").exists(), "nothing goes to the Trash");
        assert!(!home.path("home/sandbox/Sample app cache").exists());
        assert!(!home.path("home/sandbox/fresher-e2e-build").exists());
        assert_eq!(fs::read_dir(home.path("home/sandbox/Old sample downloads")).unwrap().count(), 0);
        assert_outside_untouched(&home, &outside_before);
        let report = json!({
            "events": events,
            "detail": home.engine.cleanup(cleanup).unwrap(),
            "journal": home.journal(),
            "tree": home.tree(),
        });
        check_golden("cleanup-permanent", &home.relative(&report));
    }
}
