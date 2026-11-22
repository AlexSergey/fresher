//! The Largest screen end to end, on files: the real scanner (its reader seen through the facts lens) with the
//! `trial` module, the real settings file (the smallest size listed set to 10 MB), scan cache and journal, and the real
//! `DiskCleaner` reaching only inside the sandbox folder, with a fake Trash. Proven here: what the Files list and the
//! Folders view never offer (a claimed folder, the Library, the Trash, a hidden folder, an app bundle, a link, a
//! package's inner file, a file below the minimum, a standard folder), that a pick never shows on Scan, that a pick
//! goes through the confirmed plan at High to the Trash and comes back by put back, and that a restored scan and a
//! rescan list the same. The first scan's Files page and the home's Folders page are the golden file.
//!
//! `UPDATE_GOLDEN=1 cargo nextest run -p fresher-e2e --test largest` rewrites `fixtures/golden/largest-flow.json`; the
//! diff is reviewed.

#[cfg(test)]
mod support;

#[cfg(test)]
#[path = "support/tree.rs"]
mod tree;

#[cfg(test)]
#[path = "largest_home/mod.rs"]
mod flow;

/// The paths of the flows, relative to the sandbox.
#[cfg(test)]
mod paths {
    /// The trial cache the `trial` module claims, and the file in it.
    pub(crate) const CLAIMED: &str = "home/project/fresher-trial-cache";
    pub(crate) const CLAIMED_FILE: &str = "home/project/fresher-trial-cache/blob.bin";
    pub(crate) const LIBRARY_FILE: &str = "home/Library/Application Support/Big/data.bin";
    pub(crate) const TRASH_FILE: &str = "home/.Trash/old.dmg";
    pub(crate) const HIDDEN_FILE: &str = "home/.android/avd/disk.img";
    pub(crate) const APP: &str = "home/Applications/Tool.app";
    pub(crate) const APP_FILE: &str = "home/Applications/Tool.app/Contents/Resources/big.bin";
    pub(crate) const LINK: &str = "home/link-to-big";
    pub(crate) const SMALL: &str = "home/Downloads/small.zip";
    pub(crate) const DOWNLOADS: &str = "home/Downloads";

    pub(crate) const INSTALLER: &str = "home/Downloads/Installer.dmg";
    pub(crate) const MOVIE: &str = "home/Movies/Trip.mov";
    pub(crate) const VM: &str = "home/VMs/Old.utm";
    pub(crate) const VM_DISK: &str = "home/VMs/Old.utm/Images/disk.qcow2";
    pub(crate) const ARCHIVE: &str = "home/Downloads/Photos.zip";
    pub(crate) const OLD_PROJECTS: &str = "home/Old projects";
    pub(crate) const OLD_FOLDER: &str = "home/Old projects/archive";
    pub(crate) const OLD_FILE: &str = "home/Old projects/archive/data.bin";

    /// What the Files list holds, largest first, ties in path order.
    pub(crate) const LISTED: [&str; 5] = [INSTALLER, MOVIE, ARCHIVE, OLD_FILE, VM];
}

#[cfg(test)]
mod flows {
    use std::fs;

    use fresher_core::{
        CommandError, DangerLevel, ItemOutcome, LargestPage, OWN_FILE_ID, Offer, OwnPlace, PartKind, PutBackOutcome,
        Withheld,
    };
    use serde_json::{Value, json};

    use crate::flow::Flow;
    use crate::paths::LISTED;
    use crate::tree::contents;

    /// The scanned flow home.
    pub(crate) fn scanned() -> Flow {
        let flow = Flow::new();
        flow.scan();
        flow
    }

    /// No row of the Files list lies at or below `relative`.
    pub(crate) fn assert_not_listed(flow: &Flow, relative: &str) {
        let listed = flow.listed();
        assert!(listed.iter().all(|path| !path.starts_with(relative)), "{relative} is listed: {listed:?}");
    }

    /// A pick of the node at `relative` is refused `NotOwnItem`.
    pub(crate) fn assert_not_pickable(flow: &Flow, relative: &str) {
        assert_eq!(flow.pick(relative).map(|_| ()), Err(CommandError::NotOwnItem), "{relative}");
    }

    /// The row of `place` named `name`, its offer and whether it opens.
    pub(crate) fn row(place: &OwnPlace, name: &str) -> (Offer, bool) {
        let names: Vec<&str> = place.rows.iter().map(|row| row.node.name.as_str()).collect();
        assert!(names.contains(&name), "{name}: {names:?}");
        let row = place.rows.iter().find(|row| row.node.name == name).unwrap();
        (row.offer, row.opens)
    }

    /// The Files list's rows as `(path, kind, offer, unpacked, picked)` values, for comparing two scans.
    pub(crate) fn rows(flow: &Flow, page: &LargestPage) -> Vec<Value> {
        page.rows
            .iter()
            .map(|row| {
                json!({
                    "path": flow.relative_path(&flow.node_path(row.node.id)),
                    "sizes": row.node.sizes,
                    "kind": row.kind,
                    "offer": row.offer,
                    "unpacked": row.unpacked,
                    "picked": row.picked.is_some(),
                })
            })
            .collect()
    }

    /// What Scan shows: the overview's totals and found sizes, every findings page, the map's explained part.
    pub(crate) fn scan_view(flow: &Flow) -> Value {
        let overview = flow.engine.overview().unwrap();
        let map = flow.engine.storage_map().unwrap();
        let explained =
            map.volumes[0].parts().iter().find(|part| part.kind == PartKind::Explained).map(|part| part.sizes);
        json!({
            "totals": overview.totals,
            "found": overview.found,
            "findings": flow.findings(),
            "explained": explained,
        })
    }

    /// The bytes of every file under the fake Trash, sorted by their path there, without the fake Trash's numbering.
    pub(crate) fn trashed(flow: &Flow) -> Vec<Vec<u8>> {
        let mut bytes: Vec<Vec<u8>> = contents(&flow.sandbox.trash()).into_values().collect();
        bytes.sort();
        bytes
    }

    pub(crate) fn a_standard_folder_cannot_be_picked() {
        let flow = scanned();

        let home = flow.place(None).unwrap();

        assert_eq!(row(&home, "Downloads"), (Offer::Withheld(Withheld::StandardFolder), true));
        assert_not_pickable(&flow, crate::paths::DOWNLOADS);
    }

    pub(crate) fn a_picked_row_is_never_on_scan() {
        let flow = scanned();
        let before = scan_view(&flow);

        let picked = flow.pick(crate::paths::INSTALLER).unwrap();

        assert_eq!(picked.finding().category().as_str(), OWN_FILE_ID);
        assert_eq!(scan_view(&flow), before, "Scan is the same after the pick");
    }

    pub(crate) fn a_package_in_use_moves_nothing() {
        let flow = scanned();
        let before = contents(&flow.path(crate::paths::VM));
        let picked = flow.pick(crate::paths::VM).unwrap();
        let view = flow.prepare(&[&picked]);
        flow.open_files.hold(flow.path(crate::paths::VM_DISK), "Virtual Machines");

        let (_, events) = flow.clean(&view);

        let outcomes = flow.outcomes(&view, &events);
        assert_eq!(outcomes.len(), 1);
        assert_eq!(outcomes[0].0, crate::paths::VM);
        assert!(
            matches!(&outcomes[0].1, ItemOutcome::Skipped { reason: fresher_core::SkipReason::InUse { app } }
                if app == "Virtual Machines"),
            "{outcomes:?}"
        );
        assert_eq!(contents(&flow.path(crate::paths::VM)), before, "every file of the package stays");
        assert!(flow.listed().iter().any(|path| path == crate::paths::VM), "it is still listed");
    }

    pub(crate) fn the_files_list_ranks_the_home_s_items() -> Value {
        let flow = scanned();

        let page = flow.largest();

        assert_eq!(flow.listed(), LISTED);
        assert_eq!(page.total, 5);
        assert!(page.rows.iter().all(|row| row.offer == Offer::Offered && row.picked.is_none()), "{page:?}");
        let allocated: Vec<u64> = page.rows.iter().map(|row| row.node.sizes.allocated.bytes()).collect();
        assert!(allocated.windows(2).all(|pair| pair[0] >= pair[1]), "{allocated:?}");
        assert_eq!(page.min_size.bytes(), 10_000_000);
        assert!(!page.truncated);
        json!({ "files": flow.report_of(&page), "folders": flow.report_of(&flow.place(None).unwrap()) })
    }

    pub(crate) fn a_picked_file_goes_to_the_trash_at_high_and_leaves_the_list() {
        let flow = scanned();
        let bytes = fs::read(flow.path(crate::paths::INSTALLER)).unwrap();
        let mut in_trash = trashed(&flow);
        let picked = flow.pick(crate::paths::INSTALLER).unwrap();

        let view = flow.prepare(&[&picked]);

        assert_eq!((view.summary.count, view.summary.highest), (1, DangerLevel::High));
        let (_, events) = flow.clean(&view);
        assert_eq!(flow.outcomes(&view, &events), [(crate::paths::INSTALLER.to_owned(), ItemOutcome::Moved)]);
        assert!(!flow.path(crate::paths::INSTALLER).exists());
        in_trash.push(bytes);
        in_trash.sort();
        assert_eq!(trashed(&flow), in_trash, "the file is in the Trash");
        let listed: Vec<String> =
            LISTED.iter().filter(|path| **path != crate::paths::INSTALLER).map(|p| (*p).to_owned()).collect();
        assert_eq!(flow.listed(), listed);
        let downloads = flow.place(Some(crate::paths::DOWNLOADS)).unwrap();
        assert!(downloads.rows.iter().all(|row| row.node.name != "Installer.dmg"), "{downloads:?}");
        let journal = serde_json::to_string(&flow.journal()).unwrap();
        assert!(journal.contains(OWN_FILE_ID) && journal.contains("Downloads/Installer.dmg"), "{journal}");
    }

    pub(crate) fn a_picked_file_comes_back_by_put_back() {
        let flow = scanned();
        let bytes = fs::read(flow.path(crate::paths::MOVIE)).unwrap();
        let picked = flow.pick(crate::paths::MOVIE).unwrap();
        let (cleanup, _) = flow.clean(&flow.prepare(&[&picked]));
        assert!(!flow.path(crate::paths::MOVIE).exists());

        let back = flow.engine.put_back(cleanup, None).unwrap();

        assert_eq!(back.items.len(), 1);
        assert_eq!(back.items[0].outcome, PutBackOutcome::PutBack, "{back:?}");
        assert_eq!(fs::read(flow.path(crate::paths::MOVIE)).unwrap(), bytes, "put back byte for byte");
        assert_eq!(
            contents(&flow.sandbox.trash()).keys().filter(|path| path.to_string_lossy().ends_with("Trip.mov")).count(),
            0
        );
    }

    pub(crate) fn a_picked_folder_goes_whole() {
        let flow = scanned();
        let bytes = fs::read(flow.path(crate::paths::OLD_FILE)).unwrap();
        let home = flow.place(None).unwrap();
        assert_eq!(row(&home, "Old projects"), (Offer::Offered, true));
        let old = flow.place(Some(crate::paths::OLD_PROJECTS)).unwrap();
        assert_eq!(row(&old, "archive"), (Offer::Offered, true));
        let folder = old.rows.iter().find(|row| row.node.name == "archive").unwrap();
        let picked = flow.engine.pick_own_item(folder.node.id).unwrap();

        let view = flow.prepare(&[&picked]);

        assert_eq!((view.summary.count, view.summary.highest), (1, DangerLevel::High));
        let (_, events) = flow.clean(&view);
        assert_eq!(flow.outcomes(&view, &events), [(crate::paths::OLD_FOLDER.to_owned(), ItemOutcome::Moved)]);
        assert!(!flow.path(crate::paths::OLD_FOLDER).exists());
        assert!(flow.path(crate::paths::OLD_PROJECTS).is_dir(), "the folder above stays");
        let moved = contents(&flow.sandbox.trash());
        let trashed: Vec<&Vec<u8>> = moved
            .iter()
            .filter(|(path, _)| path.to_string_lossy().ends_with("archive/data.bin"))
            .map(|(_, bytes)| bytes)
            .collect();
        assert_eq!(trashed, [&bytes], "the folder is in the Trash whole: {:?}", moved.keys());
        assert!(!flow.listed().iter().any(|path| path == crate::paths::OLD_FILE));
    }

    pub(crate) fn a_restored_scan_lists_the_same_rows() {
        let mut flow = scanned();
        let before = rows(&flow, &flow.largest());
        flow.restart();

        flow.restore();

        assert_eq!(rows(&flow, &flow.largest()), before);
    }

    pub(crate) fn a_rescan_after_put_back_lists_it_again() {
        let flow = scanned();
        let picked = flow.pick(crate::paths::MOVIE).unwrap();
        let (cleanup, _) = flow.clean(&flow.prepare(&[&picked]));
        assert!(!flow.listed().iter().any(|path| path == crate::paths::MOVIE));
        flow.engine.put_back(cleanup, None).unwrap();

        flow.rescan();

        assert_eq!(flow.listed(), LISTED);
        assert!(flow.largest().rows.iter().all(|row| row.picked.is_none()), "a scan forgets the picks");
    }
}

mod negative {
    use fresher_core::{Offer, Withheld};

    use crate::flows::{self, assert_not_listed, assert_not_pickable, row, scanned};
    use crate::paths::{
        APP, APP_FILE, CLAIMED, CLAIMED_FILE, HIDDEN_FILE, LIBRARY_FILE, LINK, SMALL, TRASH_FILE, VM_DISK,
    };

    #[test]
    fn a_claimed_folder_is_never_listed() {
        let flow = scanned();

        assert!(flow.findings().iter().any(|(_, path)| path == CLAIMED), "the trial module claims it");
        assert_not_listed(&flow, "home/project");
        let project = flow.place(Some("home/project")).unwrap();
        assert_eq!(row(&project, "fresher-trial-cache"), (Offer::Withheld(Withheld::Explained), false));
        assert_not_pickable(&flow, CLAIMED);
        let state = flow.engine.disk_state().unwrap().unwrap();
        assert_eq!(state.find(&flow.path(CLAIMED_FILE)), None, "a claimed folder is one node: nothing inside to pick");
    }

    #[test]
    fn nothing_under_the_library_the_trash_or_a_hidden_folder_is_listed() {
        let flow = scanned();

        let home = flow.place(None).unwrap();

        assert_eq!(row(&home, "Library"), (Offer::Withheld(Withheld::Library), false));
        assert_eq!(row(&home, ".Trash"), (Offer::Withheld(Withheld::Trash), false));
        assert_eq!(row(&home, ".android"), (Offer::Withheld(Withheld::Hidden), false));
        for path in [LIBRARY_FILE, TRASH_FILE, HIDDEN_FILE] {
            assert_not_listed(&flow, path);
            assert_not_pickable(&flow, path);
        }
        for folder in ["home/Library", "home/.Trash", "home/.android"] {
            assert_eq!(flow.place(Some(folder)).map(|_| ()), Err(fresher_core::CommandError::NotOwnItem), "{folder}");
        }
    }

    #[test]
    fn an_app_bundle_and_a_link_are_never_listed() {
        let flow = scanned();

        let home = flow.place(None).unwrap();
        let applications = flow.place(Some("home/Applications")).unwrap();

        let link = home.rows.iter().find(|row| row.node.name == "link-to-big");
        assert!(link.is_none_or(|link| link.offer == Offer::Withheld(Withheld::Link) && !link.opens), "{link:?}");
        assert_eq!(row(&applications, "Tool.app"), (Offer::Withheld(Withheld::App), false));
        for path in [APP, APP_FILE] {
            assert_not_listed(&flow, path);
            assert_not_pickable(&flow, path);
        }
        assert_not_listed(&flow, LINK);
        let state = flow.engine.disk_state().unwrap().unwrap();
        assert_eq!(state.find(&flow.path(LINK)), None, "the tree keeps no node of a link: nothing to pick");
        assert_eq!(flow.listed().iter().filter(|path| path.ends_with("Trip.mov")).count(), 1, "once, by its own name");
    }

    #[test]
    fn a_package_s_inner_file_is_never_listed() {
        let flow = scanned();

        let vms = flow.place(Some("home/VMs")).unwrap();

        assert_eq!(row(&vms, "Old.utm"), (Offer::Offered, false));
        assert_not_listed(&flow, VM_DISK);
        assert_not_pickable(&flow, VM_DISK);
        assert_eq!(flow.place(Some("home/VMs/Old.utm")).map(|_| ()), Err(fresher_core::CommandError::NotOwnItem));
    }

    #[test]
    fn a_file_below_the_minimum_is_not_listed() {
        let flow = scanned();

        assert_not_listed(&flow, SMALL);
        assert_eq!(flow.largest().min_size.bytes(), 10_000_000);
    }

    #[test]
    fn a_standard_folder_cannot_be_picked() {
        flows::a_standard_folder_cannot_be_picked();
    }

    #[test]
    fn a_picked_row_is_never_on_scan() {
        flows::a_picked_row_is_never_on_scan();
    }

    #[test]
    fn a_package_in_use_moves_nothing() {
        flows::a_package_in_use_moves_nothing();
    }
}

mod positive {
    use fresher_core::{ItemKind, Offer};

    use crate::flows::{self, scanned};
    use crate::paths::{ARCHIVE, INSTALLER, VM};
    use crate::support::check_golden;

    #[test]
    fn the_files_list_ranks_the_home_s_items() {
        check_golden("largest-flow", &flows::the_files_list_ranks_the_home_s_items());
    }

    #[test]
    fn a_package_is_one_row_with_its_kind() {
        let flow = scanned();

        let page = flow.largest();

        let vm = page.rows.iter().find(|row| flow.node_path(row.node.id) == flow.path(VM)).unwrap();
        assert_eq!((vm.kind, vm.offer), (ItemKind::VirtualMachine, Offer::Offered));
        assert!(vm.node.sizes.allocated.bytes() >= 12 * 1024 * 1024, "{vm:?}");
    }

    #[test]
    fn an_archive_beside_its_folder_is_pointed_out() {
        let flow = scanned();

        let page = flow.largest();

        let unpacked = |relative: &str| {
            let row = page.rows.iter().find(|row| flow.node_path(row.node.id) == flow.path(relative)).unwrap();
            (row.kind, row.unpacked.clone())
        };
        assert_eq!(unpacked(ARCHIVE), (ItemKind::Archive, Some("Photos".to_owned())));
        assert_eq!(unpacked(INSTALLER), (ItemKind::DiskImage, None));
    }

    #[test]
    fn the_folders_view_opens_through_the_home() {
        let flow = scanned();

        let home = flow.place(None).unwrap();
        let downloads = flow.place(Some("home/Downloads")).unwrap();

        assert_eq!(home.path.len(), 1);
        assert!(home.path[0].name.ends_with("home"), "{:?}", home.path[0]);
        assert_eq!(flows::row(&home, "VMs"), (Offer::Offered, true));
        let way: Vec<&str> = downloads.path.iter().skip(1).map(|node| node.name.as_str()).collect();
        assert_eq!((downloads.path.len(), way), (2, vec!["Downloads"]));
        for name in ["Installer.dmg", "Photos.zip", "small.zip"] {
            assert_eq!(flows::row(&downloads, name), (Offer::Offered, false), "{name}");
        }
        assert_eq!(flows::row(&downloads, "Photos"), (Offer::Offered, true));
    }

    #[test]
    fn a_picked_file_goes_to_the_trash_at_high_and_leaves_the_list() {
        flows::a_picked_file_goes_to_the_trash_at_high_and_leaves_the_list();
    }

    #[test]
    fn a_picked_file_comes_back_by_put_back() {
        flows::a_picked_file_comes_back_by_put_back();
    }

    #[test]
    fn a_picked_folder_goes_whole() {
        flows::a_picked_folder_goes_whole();
    }

    #[test]
    fn a_restored_scan_lists_the_same_rows() {
        flows::a_restored_scan_lists_the_same_rows();
    }

    #[test]
    fn a_rescan_after_put_back_lists_it_again() {
        flows::a_rescan_after_put_back_lists_it_again();
    }
}
