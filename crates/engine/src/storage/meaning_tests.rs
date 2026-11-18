use std::path::{Path, PathBuf};
use std::sync::Arc;

use fresher_core::{
    BundleId, CategoryDescriptor, DangerLevel, DiskState, Finding, FindingId, InstalledApp, MeaningGroup, NodeId,
    NodeKind, Removability, Sizes, StateId, SystemFacts, SystemPorts, Timestamp,
};
use fresher_testing::{FakeInstalledApps, FakeKnownFolders, FakeRunningApps, FixedDiskState, installed_app};

use crate::settings::Ignored;
use crate::storage::attribution::AppPlaces;
use crate::storage::meaning::{append_volume_parts, meaning};
use crate::storage::shares::Shares;
use crate::testing::{bytes, category, finding};

fn caches() -> CategoryDescriptor {
    category("alpha.caches", DangerLevel::Safe)
}

fn logs() -> CategoryDescriptor {
    category("alpha.logs", DangerLevel::Safe)
}

/// A state of folders with the sizes given, taken as they are.
fn state(nodes: &[(&str, u64)]) -> FixedDiskState {
    let nodes = nodes.iter().map(|&(path, size)| (PathBuf::from(path), NodeKind::Folder, bytes(size)));
    FixedDiskState::new(StateId::new(1), Timestamp::from_secs(0), nodes)
}

fn id(id: &str) -> BundleId {
    BundleId::new(id).unwrap()
}

fn app(bundle: &str, name: &str) -> InstalledApp {
    installed_app(bundle, name, "1.0").unwrap()
}

/// A published finding of `category` about `path`, for `app` when one is named.
fn found(number: u32, category: &CategoryDescriptor, path: &str, app: Option<&str>) -> Finding {
    let new = finding(category, path, Sizes::ZERO);
    let new = match app {
        Some(app) => new.for_app(id(app)),
        None => new,
    };
    Finding::new(FindingId::new(number), new, Removability::Removable)
}

/// The groups of `state` for `findings`, with `apps` installed in the home `/home`, under `ignored`.
fn groups(state: &FixedDiskState, apps: Vec<InstalledApp>, findings: &[Finding], ignored: &Ignored) -> Groups {
    let facts = SystemFacts::new(&SystemPorts {
        known_folders: Arc::new(FakeKnownFolders::home_layout(Path::new("/home"))),
        apps: Arc::new(FakeInstalledApps::new(apps)),
        running: Arc::new(FakeRunningApps::default()),
    });
    let places = AppPlaces::of(state, &facts);
    let shares = Shares::of(state, ignored);
    Groups(meaning(state, &shares, &places, findings, &state.roots()))
}

/// The groups as short lines, to compare at a glance: `app <id> <bytes>`, `  category <id> <bytes>`,
/// `  data <name> <bytes>`, `category <id> <bytes>`, `place <name> <bytes>`, `part <bytes>`.
struct Groups(Vec<MeaningGroup>);

impl Groups {
    fn lines(&self) -> Vec<String> {
        use fresher_core::PlaceName;
        let name = |name: &PlaceName| match name {
            PlaceName::Folder(name) => name.clone(),
            PlaceName::Trash => "Trash".to_owned(),
        };
        let mut lines = Vec::new();
        for group in &self.0 {
            match group {
                MeaningGroup::App { app, sizes, categories, data } => {
                    let state = if app.installed { "" } else { " (gone)" };
                    lines.push(format!("app {} {}{state} {}", app.id, app.name, sizes.allocated.bytes()));
                    for share in categories {
                        lines.push(format!("  category {} {}", share.category, share.sizes.allocated.bytes()));
                    }
                    for place in data {
                        lines.push(format!("  data {} {}", name(&place.name), place.sizes.allocated.bytes()));
                    }
                }
                MeaningGroup::Category(share) => {
                    lines.push(format!("category {} {}", share.category, share.sizes.allocated.bytes()));
                }
                MeaningGroup::Place(place) => {
                    lines.push(format!("place {} {}", name(&place.name), place.sizes.allocated.bytes()));
                }
                MeaningGroup::Part(part) => {
                    lines.push(format!("part {:?} {}", part.kind, part.sizes.allocated.bytes()));
                }
            }
        }
        lines
    }

    fn node_of_trash(&self) -> Option<NodeId> {
        self.0.iter().find_map(|group| match group {
            MeaningGroup::Place(place) if place.name == fresher_core::PlaceName::Trash => Some(place.node),
            _ => None,
        })
    }
}

mod negative {
    use std::path::{Path, PathBuf};

    use fresher_core::{InstalledApp, NodeKind, StateId, Timestamp};
    use fresher_testing::FixedDiskState;

    use super::{app, caches, found, groups, logs, state};
    use crate::settings::Ignored;
    use crate::testing::bytes;

    #[test]
    fn a_folder_named_like_an_app_is_not_its_without_the_dot() {
        let state = state(&[
            ("/home", 100),
            ("/home/Library", 100),
            ("/home/Library/Caches", 100),
            ("/home/Library/Caches/com.example.Appx", 60),
        ]);

        let groups = groups(&state, vec![app("com.example.App", "App")], &[], &Ignored::default());

        assert_eq!(groups.lines(), ["place Library 100", "part YourFolders 0"]);
    }

    #[test]
    fn a_folder_outside_the_app_data_folders_is_not_attributed() {
        let state = state(&[("/home", 100), ("/home/Documents", 70), ("/home/Documents/com.example.App", 70)]);

        let groups = groups(&state, vec![app("com.example.App", "App")], &[], &Ignored::default());

        assert_eq!(groups.lines(), ["place Documents 70", "part YourFolders 30"]);
    }

    #[test]
    fn an_app_folder_claimed_by_a_module_counts_once() {
        let state = state(&[
            ("/home", 1000),
            ("/home/Library", 600),
            ("/home/Library/Caches", 500),
            ("/home/Library/Caches/com.example.App", 300),
            ("/home/Library/Caches/com.example.App/blobs", 200),
        ])
        .with_owner(Path::new("/home/Library/Caches/com.example.App/blobs"), caches().id());
        let blobs = found(1, &caches(), "/home/Library/Caches/com.example.App/blobs", Some("com.example.App"));

        let groups = groups(&state, vec![app("com.example.App", "App")], &[blobs], &Ignored::default());

        assert_eq!(
            groups.lines(),
            [
                "app com.example.App App 300",
                "  category alpha.caches 200",
                "  data com.example.App 100",
                "place Library 300",
                "part YourFolders 400",
            ]
        );
    }

    #[test]
    fn an_ignored_finding_of_an_app_is_its_data() {
        let state = state(&[("/home", 100), ("/home/Developer", 80), ("/home/Developer/cache", 50)])
            .with_owner(Path::new("/home/Developer/cache"), caches().id());
        let cache = found(1, &caches(), "/home/Developer/cache", Some("com.example.App"));
        let ignored = Ignored { categories: vec!["alpha.caches".into()], ..Ignored::default() };

        let groups = groups(&state, vec![app("com.example.App", "App")], &[cache], &ignored);

        assert_eq!(
            groups.lines(),
            ["app com.example.App App 50", "  data cache 50", "place Developer 30", "part YourFolders 20"]
        );
    }

    #[test]
    fn a_hidden_modules_finding_of_an_app_is_its_data() {
        let state = state(&[("/home", 100), ("/home/Developer", 80), ("/home/Developer/cache", 50)])
            .with_owner(Path::new("/home/Developer/cache"), caches().id());
        let cache = found(1, &caches(), "/home/Developer/cache", Some("com.example.App"));
        let hidden = Ignored { hidden_modules: vec![caches().id().module()], ..Ignored::default() };

        let groups = groups(&state, vec![app("com.example.App", "App")], &[cache], &hidden);

        assert_eq!(
            groups.lines(),
            ["app com.example.App App 50", "  data cache 50", "place Developer 30", "part YourFolders 20"]
        );
    }

    #[test]
    fn a_hidden_modules_finding_of_no_app_is_your_folders() {
        let state = state(&[("/home", 100), ("/home/Developer", 80), ("/home/Developer/cache", 50)])
            .with_owner(Path::new("/home/Developer/cache"), caches().id());
        let cache = found(1, &caches(), "/home/Developer/cache", None);
        let hidden = Ignored { hidden_modules: vec![caches().id().module()], ..Ignored::default() };

        let groups = groups(&state, Vec::new(), &[cache], &hidden);

        assert_eq!(groups.lines(), ["place Developer 80", "part YourFolders 20"], "the cache's 50 stay in Developer");
    }

    #[test]
    fn an_ignored_finding_inside_another_apps_folder_is_that_apps_data() {
        let state = state(&[
            ("/home", 100),
            ("/home/Library", 100),
            ("/home/Library/Containers", 100),
            ("/home/Library/Containers/com.example.A", 70),
            ("/home/Library/Containers/com.example.A/b-cache", 40),
        ])
        .with_owner(Path::new("/home/Library/Containers/com.example.A/b-cache"), caches().id());
        let cache = found(1, &caches(), "/home/Library/Containers/com.example.A/b-cache", Some("com.example.B"));
        let ignored = Ignored { categories: vec!["alpha.caches".into()], ..Ignored::default() };
        let apps = vec![app("com.example.A", "A"), app("com.example.B", "B")];

        let groups = groups(&state, apps, &[cache], &ignored);

        assert_eq!(
            groups.lines(),
            ["app com.example.A A 70", "  data com.example.A 70", "place Library 30", "part YourFolders 0"],
            "the outermost place wins: B gets nothing"
        );
    }

    #[test]
    fn a_finding_whose_category_does_not_own_its_folder_adds_nothing_to_the_app() {
        let state = state(&[("/home", 100), ("/home/Logs", 40)]).with_owner(Path::new("/home/Logs"), logs().id());
        let claimed = found(1, &logs(), "/home/Logs", None);
        let lost = found(2, &caches(), "/home/Logs", Some("com.example.App"));
        let missing = found(3, &caches(), "/home/Nowhere", Some("com.example.App"));

        let groups = groups(&state, Vec::<InstalledApp>::new(), &[claimed, lost, missing], &Ignored::default());

        assert_eq!(groups.lines(), ["category alpha.logs 40", "part YourFolders 60"]);
    }

    #[test]
    fn an_empty_trash_and_empty_folders_are_not_listed() {
        let state = state(&[("/home", 10), ("/home/.Trash", 0), ("/home/Empty", 0)]);

        let groups = groups(&state, Vec::new(), &[], &Ignored::default());

        assert_eq!(groups.lines(), ["part YourFolders 10"]);
    }

    #[test]
    fn a_top_level_file_is_not_a_place_but_part_of_your_folders() {
        let nodes = [
            ("/home", NodeKind::Folder, 100),
            ("/home/Movies", NodeKind::Folder, 60),
            ("/home/archive.zip", NodeKind::File, 30),
            ("/home/Shared", NodeKind::Link, 10),
        ];
        let nodes = nodes.iter().map(|&(path, kind, size)| (PathBuf::from(path), kind, bytes(size)));
        let state = FixedDiskState::new(StateId::new(1), Timestamp::from_secs(0), nodes);

        let groups = groups(&state, Vec::new(), &[], &Ignored::default());

        assert_eq!(groups.lines(), ["place Movies 60", "part YourFolders 40"]);
    }
}

mod positive {
    use std::path::Path;

    use fresher_core::{
        ByteSize, DiskState, Finding, FindingId, InstalledApp, MapPart, MeaningGroup, MountKind, PartKind,
        Removability, Sizes, VolumeMap,
    };

    use super::{app, append_volume_parts, caches, found, groups, id, logs, state};
    use crate::settings::Ignored;
    use crate::testing::finding;

    #[test]
    fn the_volume_parts_after_the_scanned_ones_end_the_groups() {
        let part = |kind: PartKind, bytes: u64| MapPart {
            kind,
            sizes: Sizes::complete(ByteSize::new(bytes), ByteSize::new(bytes)),
        };
        let remainder = PartKind::Remainder { outside_scan: true, reasons: vec![] };
        let parts = vec![
            part(PartKind::Explained, 1),
            part(PartKind::YourFolders, 2),
            part(remainder.clone(), 3),
            part(PartKind::MacOs, 4),
            part(PartKind::SwapAndSleep, 0),
            part(PartKind::DiskOwn, 5),
            part(PartKind::Free, 6),
        ];
        let volume =
            VolumeMap::new("Data".into(), MountKind::Internal, ByteSize::new(21), parts.clone(), None, vec![], false)
                .unwrap();
        let mut groups = vec![MeaningGroup::Part(part(PartKind::YourFolders, 2))];

        append_volume_parts(&mut groups, &volume);

        let expected: Vec<MeaningGroup> = std::iter::once(part(PartKind::YourFolders, 2))
            .chain(parts[2..].iter().cloned())
            .map(MeaningGroup::Part)
            .collect();
        assert_eq!(groups, expected);
    }

    #[test]
    fn an_app_gathers_its_findings_and_its_data() {
        let state = state(&[
            ("/home", 1000),
            ("/home/Library", 700),
            ("/home/Library/Caches", 300),
            ("/home/Library/Caches/com.example.App", 250),
            ("/home/Library/Application Support", 400),
            ("/home/Library/Application Support/com.example.App.Helper", 150),
            ("/Applications", 500),
            ("/Applications/App.app", 450),
        ])
        .with_owner(Path::new("/home/Library/Caches/com.example.App"), caches().id());
        let app = InstalledApp { helpers: vec![id("com.example.App.Helper")], ..app("com.example.App", "App") };
        let cache = found(1, &caches(), "/home/Library/Caches/com.example.App", Some("com.example.App.Helper"));

        let groups = groups(&state, vec![app], &[cache], &Ignored::default());

        assert_eq!(
            groups.lines(),
            [
                "app com.example.App App 850",
                "  category alpha.caches 250",
                "  data App.app 450",
                "  data com.example.App.Helper 150",
                "place Library 300",
                "part YourFolders 350",
            ]
        );
    }

    #[test]
    fn apps_come_largest_first_then_the_categories_largest_first() {
        let state = state(&[
            ("/home", 1000),
            ("/home/Library", 1000),
            ("/home/Library/Caches", 1000),
            ("/home/Library/Caches/com.example.A", 100),
            ("/home/Library/Caches/com.example.B", 300),
            ("/home/Library/Caches/other", 200),
            ("/home/Library/Caches/logs", 250),
        ])
        .with_owner(Path::new("/home/Library/Caches/other"), caches().id())
        .with_owner(Path::new("/home/Library/Caches/logs"), logs().id());
        let findings = [found(1, &caches(), "/home/Library/Caches/other", None)];

        let groups =
            groups(&state, vec![app("com.example.A", "A"), app("com.example.B", "B")], &findings, &Ignored::default());

        assert_eq!(
            groups.lines(),
            [
                "app com.example.B B 300",
                "  data com.example.B 300",
                "app com.example.A A 100",
                "  data com.example.A 100",
                "category alpha.logs 250",
                "category alpha.caches 200",
                "place Library 150",
                "part YourFolders 0",
            ]
        );
    }

    #[test]
    fn the_trash_is_a_named_place() {
        let state = state(&[("/home", 100), ("/home/.Trash", 60), ("/home/Movies", 30)]);

        let groups = groups(&state, Vec::new(), &[], &Ignored::default());

        assert_eq!(groups.lines(), ["place Trash 60", "place Movies 30", "part YourFolders 10"]);
        assert_eq!(groups.node_of_trash(), state.find(Path::new("/home/.Trash")));
    }

    #[test]
    fn an_uninstalled_app_is_named_by_its_id() {
        let state = state(&[("/home", 100), ("/home/Caches", 40)]).with_owner(Path::new("/home/Caches"), caches().id());
        let cache = found(1, &caches(), "/home/Caches", Some("com.example.Gone"));

        let groups = groups(&state, Vec::new(), &[cache], &Ignored::default());

        assert_eq!(
            groups.lines(),
            ["app com.example.Gone com.example.Gone (gone) 40", "  category alpha.caches 40", "part YourFolders 60"]
        );
    }

    #[test]
    fn an_uninstalled_app_is_named_by_the_finding_when_it_gives_a_name() {
        let state = state(&[("/home", 100), ("/home/Caches", 40)]).with_owner(Path::new("/home/Caches"), caches().id());
        let named = finding(&caches(), "/home/Caches", Sizes::ZERO).for_app_named(id("com.example.Gone"), "Gone App");
        let cache = Finding::new(FindingId::new(1), named, Removability::Removable);

        let groups = groups(&state, Vec::new(), &[cache], &Ignored::default());

        assert_eq!(
            groups.lines(),
            ["app com.example.Gone Gone App (gone) 40", "  category alpha.caches 40", "part YourFolders 60"]
        );
    }

    #[test]
    fn at_most_eight_folders_of_a_root_are_listed() {
        let names: Vec<String> = (1..=10).map(|size| format!("/home/f{size:02}")).collect();
        let mut nodes: Vec<(&str, u64)> = vec![("/home", 100)];
        nodes.extend(names.iter().zip(1..).map(|(name, size)| (name.as_str(), size)));

        let groups = groups(&state(&nodes), Vec::new(), &[], &Ignored::default());

        let lines = groups.lines();
        assert_eq!(lines.len(), 9);
        assert_eq!(lines[0], "place f10 10");
        assert_eq!(lines[7], "place f03 3");
        assert_eq!(lines[8], "part YourFolders 48", "the two smallest and the root's own bytes");
    }
}

mod property {
    use std::path::{Path, PathBuf};
    use std::sync::Arc;

    use fresher_core::{
        ByteSize, Completeness, DangerLevel, Finding, FindingId, KnownFolder, MeaningGroup, NodeKind, Removability,
        Sizes, StateId, SystemFacts, SystemPorts, Timestamp,
    };
    use fresher_testing::{FakeInstalledApps, FakeKnownFolders, FakeRunningApps, FixedDiskState, installed_app};
    use proptest::prelude::*;

    use super::id;
    use crate::settings::Ignored;
    use crate::storage::attribution::AppPlaces;
    use crate::storage::meaning::meaning;
    use crate::storage::shares::Shares;
    use crate::testing::{category, finding};

    const MOST: usize = 20;
    const CATEGORIES: [&str; 3] = ["alpha.caches", "alpha.logs", "alpha.data"];

    /// A random forest of `count` folders named `n<i>`: folder `i` hangs under `parents[i] % (i + 1)`, or is a root
    /// when that is `i`; its sizes are its own plus its children's.
    fn forest(count: usize, parents: &[usize], own: &[u64], partial: &[bool]) -> (Vec<PathBuf>, Vec<Sizes>) {
        let mut paths: Vec<PathBuf> = Vec::with_capacity(count);
        let mut parent_of: Vec<Option<usize>> = Vec::with_capacity(count);
        for (index, &chosen) in parents.iter().take(count).enumerate() {
            let parent = chosen % (index + 1);
            if parent == index {
                paths.push(PathBuf::from(format!("/n{index}")));
                parent_of.push(None);
            } else {
                paths.push(paths[parent].join(format!("n{index}")));
                parent_of.push(Some(parent));
            }
        }
        let mut sizes: Vec<Sizes> = (0..count)
            .map(|index| Sizes {
                completeness: if partial[index] { Completeness::Partial } else { Completeness::Complete },
                ..Sizes::new(ByteSize::new(own[index]), ByteSize::new(own[index] * 2), ByteSize::new(own[index]))
            })
            .collect();
        for index in (0..count).rev() {
            if let Some(parent) = parent_of[index] {
                sizes[parent] = sizes[parent].checked_add(sizes[index]).unwrap();
            }
        }
        (paths, sizes)
    }

    fn bytes(sizes: Sizes) -> [u64; 3] {
        [sizes.logical.bytes(), sizes.allocated.bytes(), sizes.reclaimable.bytes()]
    }

    fn add(a: [u64; 3], b: [u64; 3]) -> [u64; 3] {
        [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
    }

    /// Every leaf of the groups: an app's categories and data, a category, a place, a part.
    fn leaves(groups: &[MeaningGroup]) -> [u64; 3] {
        groups.iter().fold([0; 3], |total, group| match group {
            MeaningGroup::App { categories, data, sizes, .. } => {
                let inside = categories.iter().map(|share| share.sizes).chain(data.iter().map(|place| place.sizes));
                let inside = inside.fold([0; 3], |sum, sizes| add(sum, bytes(sizes)));
                assert_eq!(inside, bytes(*sizes), "an app's size is its leaves'");
                add(total, inside)
            }
            MeaningGroup::Category(share) => add(total, bytes(share.sizes)),
            MeaningGroup::Place(place) => add(total, bytes(place.sizes)),
            MeaningGroup::Part(part) => add(total, bytes(part.sizes)),
        })
    }

    proptest! {
        #[test]
        fn the_leaves_of_the_groups_add_up_to_the_roots(
            count in 1_usize..=MOST,
            parents in prop::collection::vec(0_usize..1000, MOST),
            own in prop::collection::vec(0_u64..1000, MOST),
            partial in prop::collection::vec(prop::bool::weighted(0.1), MOST),
            owners in prop::collection::vec(prop::option::weighted(0.4, 0_usize..3), MOST),
            apps_of_findings in prop::collection::vec(prop::option::weighted(0.6, 0_usize..MOST + 2), MOST),
            installed in prop::collection::vec(prop::bool::weighted(0.5), MOST),
            folders in prop::collection::vec(0_usize..MOST, 3),
            bundle in 0_usize..MOST,
            ignored_categories in prop::collection::vec(prop::bool::weighted(0.3), 3),
            ignored_paths in prop::collection::vec(prop::bool::weighted(0.1), MOST),
            strays in prop::collection::vec((0_usize..MOST + 2, 0_usize..3, 0_usize..MOST + 2), 0..6),
        ) {
            let (paths, sizes) = forest(count, &parents, &own, &partial);
            let nodes = paths.iter().zip(&sizes).map(|(path, &sizes)| (path.clone(), NodeKind::Folder, sizes));
            let mut state = FixedDiskState::new(StateId::new(1), Timestamp::from_secs(0), nodes);
            let mut findings = Vec::new();
            for (index, (path, owner)) in paths.iter().zip(&owners).enumerate() {
                let Some(owner) = owner else { continue };
                let descriptor = category(CATEGORIES[*owner], DangerLevel::Safe);
                state = state.with_owner(path, descriptor.id());
                let mut new = finding(&descriptor, path.clone(), Sizes::ZERO);
                if let Some(app) = apps_of_findings[index] {
                    new = new.for_app(id(&format!("n{app}")));
                }
                findings.push(Finding::new(FindingId::new(u32::try_from(index).unwrap() + 1), new, Removability::Removable));
            }
            // Findings that may have lost their claim: another category than the node's owner, a node nobody owns,
            // a path with no node, or a second finding of an app at a node another one already counts.
            for (number, &(at, owner, app)) in (1000_u32..).zip(&strays) {
                let path = paths.get(at).cloned().unwrap_or_else(|| PathBuf::from(format!("/nowhere/n{at}")));
                let new = finding(&category(CATEGORIES[owner], DangerLevel::Safe), path, Sizes::ZERO);
                let new = new.for_app(id(&format!("n{app}")));
                findings.push(Finding::new(FindingId::new(number), new, Removability::Removable));
            }
            // Apps named after the folders, so the children of the app data folders are theirs.
            let apps = (0..MOST).filter(|&app| installed[app]).map(|app| {
                let mut app_of = installed_app(&format!("n{app}"), &format!("App {app}"), "1").unwrap();
                if app == bundle % count {
                    app_of.path.clone_from(&paths[app]);
                }
                app_of
            });
            let at = |index: usize| paths[index % count].clone();
            let known = FakeKnownFolders::new()
                .with(KnownFolder::AppCaches, at(folders[0]))
                .with(KnownFolder::Containers, at(folders[1]))
                .with(KnownFolder::Trash, at(folders[2]));
            let facts = SystemFacts::new(&SystemPorts {
                known_folders: Arc::new(known),
                apps: Arc::new(FakeInstalledApps::new(apps.collect())),
                running: Arc::new(FakeRunningApps::default()),
            });
            let ignored = Ignored {
                categories: CATEGORIES.iter().zip(&ignored_categories).filter(|&(_, &on)| on).map(|(id, _)| (*id).to_owned()).collect(),
                paths: paths.iter().zip(&ignored_paths).filter(|&(_, &on)| on).map(|(path, _)| path.clone()).collect(),
                ..Ignored::default()
            };
            let places = AppPlaces::of(&state, &facts);
            let shares = Shares::of(&state, &ignored);

            let groups = meaning(&state, &shares, &places, &findings, &fresher_core::DiskState::roots(&state));

            let roots = paths.iter().zip(&sizes).filter(|(path, _)| path.parent() == Some(Path::new("/")));
            let whole = roots.fold([0; 3], |total, (_, &sizes)| add(total, bytes(sizes)));
            prop_assert_eq!(leaves(&groups), whole);
            prop_assert!(matches!(groups.last(), Some(MeaningGroup::Part(_))), "your folders come last");
            // An app counts a category only for a finding of its own in that category.
            let named: std::collections::HashSet<(String, fresher_core::CategoryId)> = findings
                .iter()
                .filter_map(|found| Some((found.finding().app()?.folded().to_owned(), found.finding().category())))
                .collect();
            for group in &groups {
                if let MeaningGroup::App { app, categories, .. } = group {
                    for share in categories {
                        prop_assert!(named.contains(&(app.id.folded().to_owned(), share.category)), "{}", app.id);
                    }
                }
            }
        }
    }
}
