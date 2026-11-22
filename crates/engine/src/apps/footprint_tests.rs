use std::collections::BTreeSet;
use std::path::Path;

use fresher_core::{Access, FootprintKind, InstalledApp, KeptReason, NodeKind, StateId, Timestamp};
use fresher_testing::{FakeKnownFolders, FixedDiskState, installed_app, signed_by};

use super::{FootprintCx, footprint};
use crate::testing::bytes;

const TEAM: &str = "ABCDE12345";

/// A state of `entries` (paths under `/home/Library`) of one byte each, with every folder that holds them.
fn state(entries: &[(&str, NodeKind)]) -> FixedDiskState {
    let mut folders = BTreeSet::new();
    let mut leaves = Vec::new();
    for &(relative, kind) in entries {
        let path = Path::new("/home/Library").join(relative);
        folders.extend(path.ancestors().skip(1).filter(|at| at.starts_with("/home")).map(Path::to_path_buf));
        leaves.push((path, kind, bytes(1)));
    }
    let leaves: Vec<_> = leaves.into_iter().filter(|(path, _, _)| !folders.contains(path)).collect();
    let folders = folders.into_iter().map(|path| (path, NodeKind::Folder, bytes(1)));
    FixedDiskState::new(StateId::new(1), Timestamp::from_secs(0), folders.chain(leaves))
}

/// A state of the folders `names` under `/home/Library`.
fn of_folders(names: &[&str]) -> FixedDiskState {
    let entries: Vec<(&str, NodeKind)> = names.iter().map(|&name| (name, NodeKind::Folder)).collect();
    state(&entries)
}

fn app(id: &str, name: &str) -> InstalledApp {
    installed_app(id, name, "1.0").unwrap()
}

fn team(app: InstalledApp) -> InstalledApp {
    signed_by(app, TEAM).unwrap()
}

fn editor() -> InstalledApp {
    app("com.example.Editor", "Editor")
}

/// `editor()` at another path: a second copy of the same app.
fn copy() -> InstalledApp {
    InstalledApp { path: "/home/Applications/Editor.app".into(), ..editor() }
}

const GRANTED: FootprintCx<'static> = FootprintCx { other_apps_data: Access::Granted, ignored: &[], complete: true };

type Found = (FootprintKind, String, Option<KeptReason>);

/// The parts of `installed[at]` in `state`, as (kind, path under `/home/Library`, kept reason).
fn parts(state: &FixedDiskState, installed: &[InstalledApp], at: usize, cx: &FootprintCx<'_>) -> Vec<Found> {
    let folders = FakeKnownFolders::home_layout("/home");
    footprint(state, &folders, installed, at, cx)
        .into_iter()
        .map(|part| {
            let relative = part.path.strip_prefix("/home/Library").unwrap().to_string_lossy().into_owned();
            (part.kind, relative, part.kept)
        })
        .collect()
}

fn found(kind: FootprintKind, path: &str, kept: Option<KeptReason>) -> Found {
    (kind, path.to_owned(), kept)
}

mod negative {
    use std::path::{Path, PathBuf};

    use fresher_core::{Access, BundleId, Completeness, FootprintKind, KeptReason, NodeKind, NotScanned, Sizes};
    use fresher_testing::FixedDiskState;

    use super::{FootprintCx, GRANTED, TEAM, app, copy, editor, found, of_folders, parts, state, team};

    #[test]
    fn a_look_alike_id_is_not_a_part() {
        let state = of_folders(&["Caches/com.example.Editorx", "Application Support/com.example.Editorx"]);

        assert_eq!(parts(&state, &[editor()], 0, &GRANTED), []);
    }

    #[test]
    fn a_vendor_folder_is_not_a_part() {
        let state = of_folders(&["Caches/com.example", "Application Support/com.example", "Logs/com"]);

        assert_eq!(parts(&state, &[editor()], 0, &GRANTED), []);
    }

    #[test]
    fn a_helper_s_folder_of_another_app_is_not_a_part() {
        let installed = [app("com.example.Suite", "Suite"), app("com.example.Suite.Writer", "Writer")];
        let state = state(&[
            ("Caches/com.example.Suite.Writer", NodeKind::Folder),
            ("LaunchAgents/com.example.Suite.Writer.agent.plist", NodeKind::File),
        ]);

        assert_eq!(parts(&state, &installed, 0, &GRANTED), []);
        assert_eq!(
            parts(&state, &installed, 1, &GRANTED),
            [
                found(FootprintKind::Caches, "Caches/com.example.Suite.Writer", None),
                found(FootprintKind::LaunchAgents, "LaunchAgents/com.example.Suite.Writer.agent.plist", None),
            ]
        );
    }

    #[test]
    fn a_name_without_the_shape_is_not_a_part() {
        let state = state(&[
            ("Preferences/com.example.Editor.plist.bak", NodeKind::File),
            ("LaunchAgents/com.example.Editorx.plist", NodeKind::File),
            ("LaunchAgents/com.example.Editor", NodeKind::File),
            ("Saved Application State/com.example.Editor", NodeKind::Folder),
            ("Caches/com.example.Editor.ShipIt", NodeKind::Folder),
            ("Preferences/.plist", NodeKind::File),
            ("Group Containers/ABCDE12345", NodeKind::Folder),
            ("Group Containers/ABCDE12345.", NodeKind::Folder),
        ]);

        assert_eq!(parts(&state, &[team(editor())], 0, &GRANTED), []);
    }

    #[test]
    fn an_index_past_the_list_has_no_part() {
        let state = of_folders(&["Caches/com.example.Editor"]);

        assert_eq!(parts(&state, &[editor()], 1, &GRANTED), []);
    }

    #[test]
    fn a_second_copy_keeps_the_data_shared() {
        let state = of_folders(&["Caches/com.example.Editor"]);
        let shared = [found(FootprintKind::Caches, "Caches/com.example.Editor", Some(KeptReason::Shared))];

        assert_eq!(parts(&state, &[editor(), copy()], 0, &GRANTED), shared);
        assert_eq!(parts(&state, &[editor(), copy()], 1, &GRANTED), shared);
    }

    #[test]
    fn a_helper_id_another_app_has_keeps_the_data_shared() {
        let mut suite = app("com.example.Suite", "Suite");
        suite.helpers = vec![BundleId::new("com.example.Editor").unwrap()];
        let state = of_folders(&["Caches/com.example.Editor"]);

        assert_eq!(
            parts(&state, &[editor(), suite], 0, &GRANTED),
            [found(FootprintKind::Caches, "Caches/com.example.Editor", Some(KeptReason::Shared))]
        );
    }

    #[test]
    fn a_group_container_of_a_shared_team_is_kept() {
        let installed = [team(editor()), team(app("com.example.Viewer", "Viewer"))];
        let state = of_folders(&["Group Containers/ABCDE12345.shared"]);
        let kept =
            [found(FootprintKind::GroupContainer, "Group Containers/ABCDE12345.shared", Some(KeptReason::TeamShared))];

        assert_eq!(parts(&state, &installed, 0, &GRANTED), kept);
        assert_eq!(parts(&state, &installed, 1, &GRANTED), kept);
    }

    #[test]
    fn an_app_without_a_team_keeps_its_group_containers() {
        let state = of_folders(&["Group Containers/group.com.example.Editor", "Group Containers/ABCDE12345.x"]);

        assert_eq!(
            parts(&state, &[editor()], 0, &GRANTED),
            [found(
                FootprintKind::GroupContainer,
                "Group Containers/group.com.example.Editor",
                Some(KeptReason::TeamShared)
            )]
        );
    }

    #[test]
    fn a_group_container_of_another_prefix_is_kept_even_with_a_team() {
        let state = of_folders(&["Group Containers/group.com.example.Editor"]);

        assert_eq!(
            parts(&state, &[team(editor())], 0, &GRANTED),
            [found(
                FootprintKind::GroupContainer,
                "Group Containers/group.com.example.Editor",
                Some(KeptReason::TeamShared)
            )]
        );
    }

    #[test]
    fn a_link_is_kept() {
        let state = state(&[("Caches/com.example.Editor", NodeKind::Link)]);

        assert_eq!(
            parts(&state, &[editor()], 0, &GRANTED),
            [found(FootprintKind::Caches, "Caches/com.example.Editor", Some(KeptReason::Link))]
        );
    }

    #[test]
    fn a_container_without_other_apps_data_is_kept() {
        let team_folder = format!("Group Containers/{TEAM}.editor");
        let state = of_folders(&["Containers/com.example.Editor", &team_folder]);

        for access in [Access::Denied, Access::NotAsked, Access::Unknown] {
            let cx = FootprintCx { other_apps_data: access, ignored: &[], complete: true };
            assert_eq!(
                parts(&state, &[team(editor())], 0, &cx),
                [
                    found(FootprintKind::Container, "Containers/com.example.Editor", Some(KeptReason::NeedsPermission)),
                    found(FootprintKind::GroupContainer, &team_folder, Some(KeptReason::NeedsPermission)),
                ],
                "{access:?}"
            );
        }
    }

    #[test]
    fn a_folder_not_read_in_full_is_kept() {
        let path = Path::new("/home/Library/Caches/com.example.Editor");
        let unread = of_folders(&["Caches/com.example.Editor"]).with_not_scanned(path, NotScanned::Avoided);
        let partial = FixedDiskState::new(
            fresher_core::StateId::new(1),
            fresher_core::Timestamp::from_secs(0),
            [
                ("/home/Library".into(), NodeKind::Folder, Sizes::ZERO),
                ("/home/Library/Caches".into(), NodeKind::Folder, Sizes::ZERO),
                (path.to_path_buf(), NodeKind::Folder, Sizes { completeness: Completeness::Partial, ..Sizes::ZERO }),
            ],
        );
        let kept = [found(FootprintKind::Caches, "Caches/com.example.Editor", Some(KeptReason::NotScanned))];

        assert_eq!(parts(&unread, &[editor()], 0, &GRANTED), kept);
        assert_eq!(parts(&partial, &[editor()], 0, &GRANTED), kept);
    }

    #[test]
    fn an_ignored_folder_is_kept() {
        let state = of_folders(&["Caches/com.example.Editor/inner"]);
        let kept = [found(FootprintKind::Caches, "Caches/com.example.Editor", Some(KeptReason::Ignored))];

        for ignored in [
            "/home/Library/Caches/com.example.Editor",
            "/home/Library/Caches/com.example.Editor/inner",
            "/home/Library",
        ] {
            let ignored = [PathBuf::from(ignored)];
            let cx = FootprintCx { other_apps_data: Access::Granted, ignored: &ignored, complete: true };
            assert_eq!(parts(&state, &[editor()], 0, &cx), kept, "{ignored:?}");
        }
    }

    #[test]
    fn every_part_is_kept_when_the_installed_list_is_not_complete() {
        let state = of_folders(&["Caches/com.example.Editor", "Application Support/com.example.Editor"]);
        let cx = FootprintCx { other_apps_data: Access::Granted, ignored: &[], complete: false };

        assert_eq!(
            parts(&state, &[editor()], 0, &cx),
            [
                found(FootprintKind::Caches, "Caches/com.example.Editor", Some(KeptReason::Shared)),
                found(FootprintKind::Support, "Application Support/com.example.Editor", Some(KeptReason::Shared)),
            ]
        );
    }

    #[test]
    fn a_part_named_only_by_a_foreign_helper_is_kept() {
        let mut owner = editor();
        owner.helpers = vec![BundleId::new("org.updates.Framework").unwrap()];
        let state = of_folders(&["Caches/org.updates.Framework", "Caches/com.example.Editor"]);

        assert_eq!(
            parts(&state, &[owner], 0, &GRANTED),
            [
                found(FootprintKind::Caches, "Caches/com.example.Editor", None),
                found(FootprintKind::Caches, "Caches/org.updates.Framework", Some(KeptReason::Shared)),
            ]
        );
    }

    #[test]
    fn the_first_reason_in_order_is_told() {
        let state = state(&[("Caches/com.example.Editor", NodeKind::Link)]);

        assert_eq!(
            parts(&state, &[editor(), copy()], 0, &GRANTED),
            [found(FootprintKind::Caches, "Caches/com.example.Editor", Some(KeptReason::Shared))]
        );
    }
}

mod positive {
    use std::path::Path;

    use fresher_core::{BundleId, DiskState, FootprintKind, NodeKind, Timestamp};
    use fresher_testing::{FakeKnownFolders, signed_by};

    use super::{GRANTED, TEAM, app, editor, footprint, found, parts, state, team};
    use crate::testing::bytes;

    #[test]
    fn each_place_of_the_table_is_found() {
        let team_folder = format!("Group Containers/{TEAM}.editor");
        let state = state(&[
            ("Caches/com.example.Editor", NodeKind::Folder),
            ("Logs/com.example.Editor", NodeKind::Folder),
            ("Saved Application State/com.example.Editor.savedState", NodeKind::Folder),
            ("LaunchAgents/com.example.Editor.plist", NodeKind::File),
            ("HTTPStorages/com.example.Editor", NodeKind::Folder),
            ("HTTPStorages/com.example.Editor.binarycookies", NodeKind::File),
            ("WebKit/com.example.Editor", NodeKind::Folder),
            ("Cookies/com.example.Editor.binarycookies", NodeKind::File),
            ("Preferences/com.example.Editor.plist", NodeKind::File),
            ("Application Support/com.example.Editor", NodeKind::Folder),
            ("Containers/com.example.Editor", NodeKind::Folder),
            (team_folder.as_str(), NodeKind::Folder),
            ("Caches/com.example.Other", NodeKind::Folder),
        ]);

        let part = |kind, path: &str| found(kind, path, None);
        assert_eq!(
            parts(&state, &[team(editor())], 0, &GRANTED),
            [
                part(FootprintKind::Caches, "Caches/com.example.Editor"),
                part(FootprintKind::Logs, "Logs/com.example.Editor"),
                part(FootprintKind::SavedState, "Saved Application State/com.example.Editor.savedState"),
                part(FootprintKind::LaunchAgents, "LaunchAgents/com.example.Editor.plist"),
                part(FootprintKind::WebData, "HTTPStorages/com.example.Editor"),
                part(FootprintKind::WebData, "HTTPStorages/com.example.Editor.binarycookies"),
                part(FootprintKind::WebData, "WebKit/com.example.Editor"),
                part(FootprintKind::WebData, "Cookies/com.example.Editor.binarycookies"),
                part(FootprintKind::Settings, "Preferences/com.example.Editor.plist"),
                part(FootprintKind::Support, "Application Support/com.example.Editor"),
                part(FootprintKind::Container, "Containers/com.example.Editor"),
                part(FootprintKind::GroupContainer, &team_folder),
            ]
        );
    }

    #[test]
    fn a_launch_agent_with_a_dotted_name_is_found() {
        let state = state(&[
            ("LaunchAgents/com.example.Editor.updater.plist", NodeKind::File),
            ("LaunchAgents/COM.EXAMPLE.EDITOR.PLIST", NodeKind::File),
        ]);

        assert_eq!(
            parts(&state, &[editor()], 0, &GRANTED),
            [
                found(FootprintKind::LaunchAgents, "LaunchAgents/COM.EXAMPLE.EDITOR.PLIST", None),
                found(FootprintKind::LaunchAgents, "LaunchAgents/com.example.Editor.updater.plist", None),
            ]
        );
    }

    #[test]
    fn a_group_container_of_the_app_s_own_team_is_found() {
        let folder = format!("Group Containers/{TEAM}.com.example.shared");
        let state = state(&[(folder.as_str(), NodeKind::Folder)]);
        let other_team = signed_by(app("com.example.Viewer", "Viewer"), "ZYXWV98765").unwrap();

        assert_eq!(
            parts(&state, &[team(editor()), other_team], 0, &GRANTED),
            [found(FootprintKind::GroupContainer, &folder, None)]
        );
    }

    #[test]
    fn a_helper_s_own_folder_is_its_app_s() {
        let mut owner = editor();
        owner.helpers = vec![BundleId::new("com.example.Editor.Helper").unwrap()];
        let state = state(&[
            ("Caches/com.example.Editor.Helper", NodeKind::Folder),
            ("Preferences/com.example.Editor.Helper.plist", NodeKind::File),
        ]);

        assert_eq!(
            parts(&state, &[owner], 0, &GRANTED),
            [
                found(FootprintKind::Caches, "Caches/com.example.Editor.Helper", None),
                found(FootprintKind::Settings, "Preferences/com.example.Editor.Helper.plist", None),
            ]
        );
    }

    #[test]
    fn a_part_carries_its_node_sizes_and_change() {
        let path = Path::new("/home/Library/Caches/com.example.Editor");
        let state =
            state(&[("Caches/com.example.Editor", NodeKind::Folder)]).with_modified(path, Timestamp::from_secs(42));

        let parts = footprint(&state, &FakeKnownFolders::home_layout("/home"), &[editor()], 0, &GRANTED);

        assert_eq!(parts.len(), 1);
        assert_eq!(parts[0].path, path);
        assert_eq!(parts[0].sizes, bytes(1));
        assert_eq!(parts[0].modified, Some(Timestamp::from_secs(42)));
        assert_eq!(Some(parts[0].node), state.find(path));
    }
}
