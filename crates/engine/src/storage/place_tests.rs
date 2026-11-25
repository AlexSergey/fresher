use std::path::{Path, PathBuf};

use fresher_core::{
    ByteSize, CategoryId, Descent, DiskState, ModuleId, NodeId, NodeKind, PlaceView, Sizes, StateId, Timestamp,
};
use fresher_testing::FixedDiskState;

use crate::settings::Ignored;
use crate::storage::place::place_view;
use crate::storage::shares::Shares;

fn category(id: &'static str) -> CategoryId {
    CategoryId::new(ModuleId::new("demo").unwrap(), id).unwrap()
}

fn sizes(bytes: u64) -> Sizes {
    Sizes::complete(ByteSize::new(bytes), ByteSize::new(bytes))
}

/// A state of the nodes given, their sizes taken as they are.
fn state(nodes: &[(&str, NodeKind, Sizes)]) -> FixedDiskState {
    let nodes = nodes.iter().map(|&(path, kind, sizes)| (PathBuf::from(path), kind, sizes));
    FixedDiskState::new(StateId::new(1), Timestamp::from_secs(0), nodes)
}

/// A state of folders with the sizes given.
fn folders(nodes: &[(&str, u64)]) -> FixedDiskState {
    let nodes: Vec<_> = nodes.iter().map(|&(path, bytes)| (path, NodeKind::Folder, sizes(bytes))).collect();
    state(&nodes)
}

fn node(state: &FixedDiskState, path: &str) -> NodeId {
    state.find(Path::new(path)).unwrap()
}

/// The first page of up to 50 rows of `path`, nothing ignored.
fn open(state: &FixedDiskState, path: &str) -> PlaceView {
    let shares = Shares::of(state, &Ignored::default(), &crate::withheld::Withheld::offered());
    place_view(state, &shares, node(state, path), Descent::Through, 0, 50).unwrap()
}

fn path_names(view: &PlaceView) -> Vec<String> {
    view.path.iter().map(|node| node.name.clone()).collect()
}

fn row_names(view: &PlaceView) -> Vec<String> {
    view.rows.iter().map(|row| row.node.name.clone()).collect()
}

mod negative {
    use fresher_core::{Descent, NodeId, NodeKind, NotScanned, StateId, Unreadable};

    use super::{folders, node, open, path_names, row_names, sizes, state};
    use crate::settings::Ignored;
    use crate::storage::place::place_view;
    use crate::storage::shares::Shares;

    #[test]
    fn a_folder_with_no_dominant_child_does_not_descend() {
        let state = folders(&[("/home", 100), ("/home/a", 50), ("/home/b", 50)]);

        let view = open(&state, "/home");

        assert_eq!(path_names(&view), ["home"]);
        assert_eq!(row_names(&view), ["a", "b"]);
    }

    #[test]
    fn a_child_just_under_the_share_does_not_open() {
        let state = folders(&[("/home", 100), ("/home/a", 89), ("/home/b", 11)]);

        assert_eq!(path_names(&open(&state, "/home")), ["home"]);
    }

    #[test]
    fn the_descent_stops_at_an_unreadable_folder() {
        let locked = NotScanned::Unreadable(Unreadable::NoPermission);
        let state = folders(&[("/home", 100), ("/home/Library", 95), ("/home/Library/locked", 95)])
            .with_not_scanned(std::path::Path::new("/home/Library/locked"), locked);

        let view = open(&state, "/home");

        assert_eq!(path_names(&view), ["home", "Library"]);
        assert_eq!(row_names(&view), ["locked"]);
    }

    #[test]
    fn the_descent_stops_at_an_avoided_folder() {
        let state = folders(&[("/home", 100), ("/home/Library", 95), ("/home/Library/Mobile Documents", 95)])
            .with_not_scanned(std::path::Path::new("/home/Library/Mobile Documents"), NotScanned::Avoided);

        let view = open(&state, "/home");

        assert_eq!(path_names(&view), ["home", "Library"]);
        assert_eq!(row_names(&view), ["Mobile Documents"]);
    }

    #[test]
    fn the_descent_stops_at_a_folder_on_another_volume() {
        let state = folders(&[("/home", 100), ("/home/Backup", 95)])
            .with_not_scanned(std::path::Path::new("/home/Backup"), NotScanned::OtherVolume);

        let view = open(&state, "/home");

        assert_eq!(path_names(&view), ["home"]);
        assert_eq!(row_names(&view), ["Backup"]);
    }

    #[test]
    fn a_folder_of_zero_size_does_not_descend() {
        let state = folders(&[("/home", 0), ("/home/Empty", 0)]);

        let view = open(&state, "/home");

        assert_eq!(path_names(&view), ["home"]);
        assert_eq!(row_names(&view), ["Empty"]);
    }

    #[test]
    fn a_root_asked_for_no_rows_is_its_own_path_and_nothing_more() {
        let state = folders(&[("/home", 100), ("/home/Library", 95)]);
        let shares = Shares::of(&state, &Ignored::default(), &crate::withheld::Withheld::offered());

        let view = place_view(&state, &shares, node(&state, "/home"), Descent::Stay, 0, 0).unwrap();

        assert_eq!(path_names(&view), ["home"]);
        assert_eq!(row_names(&view), [] as [String; 0]);
    }

    #[test]
    fn the_descent_stops_at_a_file() {
        let state = state(&[("/home", NodeKind::Folder, sizes(100)), ("/home/movie.mov", NodeKind::File, sizes(99))]);

        let view = open(&state, "/home");

        assert_eq!(path_names(&view), ["home"]);
        assert_eq!(row_names(&view), ["movie.mov"]);
    }

    #[test]
    fn the_descent_stops_after_eight_levels() {
        let chain: Vec<String> = (0..12)
            .scan(String::new(), |path, level| {
                path.push_str("/c");
                path.push_str(&level.to_string());
                Some(path.clone())
            })
            .collect();
        let nodes: Vec<(&str, u64)> = chain.iter().map(|path| (path.as_str(), 100)).collect();
        let state = folders(&nodes);

        let view = open(&state, "/c0");

        assert_eq!(path_names(&view), ["c0", "c1", "c2", "c3", "c4", "c5", "c6", "c7", "c8"]);
        assert_eq!(row_names(&view), ["c9"]);
    }

    #[test]
    fn a_next_page_does_not_descend_again() {
        let state = folders(&[("/home", 100), ("/home/Library", 95), ("/home/Library/a", 95), ("/home/x", 5)]);
        let shares = Shares::of(&state, &Ignored::default(), &crate::withheld::Withheld::offered());

        let view = place_view(&state, &shares, node(&state, "/home"), Descent::Through, 1, 50).unwrap();

        assert_eq!(path_names(&view), ["home"]);
        assert_eq!(row_names(&view), ["x"]);
        assert_eq!((view.offset, view.total), (1, 2));
    }

    #[test]
    fn a_breadcrumb_step_does_not_descend_again() {
        let state = folders(&[("/home", 100), ("/home/Library", 95), ("/home/Library/a", 95), ("/home/x", 5)]);
        let shares = Shares::of(&state, &Ignored::default(), &crate::withheld::Withheld::offered());

        let view = place_view(&state, &shares, node(&state, "/home"), Descent::Stay, 0, 50).unwrap();

        assert_eq!(path_names(&view), ["home"]);
        assert_eq!(row_names(&view), ["Library", "x"]);
    }

    #[test]
    fn a_node_of_another_state_is_none() {
        let state = folders(&[("/home", 100)]);
        let shares = Shares::of(&state, &Ignored::default(), &crate::withheld::Withheld::offered());

        assert_eq!(place_view(&state, &shares, NodeId::new(StateId::new(2), 0), Descent::Through, 0, 50), None);
    }

    #[test]
    fn a_file_has_no_rows() {
        let state = state(&[("/home", NodeKind::Folder, sizes(100)), ("/home/movie.mov", NodeKind::File, sizes(90))]);

        let view = open(&state, "/home/movie.mov");

        assert_eq!(path_names(&view), ["movie.mov"]);
        assert_eq!(row_names(&view), [] as [&str; 0]);
        assert_eq!((view.offset, view.total), (0, 0));
    }
}

mod positive {
    use std::path::Path;

    use fresher_core::{ByteSize, Completeness, NodeKind, Sizes};

    use super::{category, folders, open, path_names, row_names, sizes, state};

    #[test]
    fn a_folder_that_is_almost_one_child_opens_through_it() {
        let state = folders(&[
            ("/home/Library", 100),
            ("/home/Library/Containers", 95),
            ("/home/Library/Containers/com.example.app", 95),
            ("/home/Library/Containers/com.example.app/Data", 60),
            ("/home/Library/Containers/com.example.app/Caches", 35),
            ("/home/Library/Preferences", 5),
        ]);

        let view = open(&state, "/home/Library");

        assert_eq!(path_names(&view), ["Library", "Containers", "com.example.app"]);
        assert_eq!(row_names(&view), ["Data", "Caches"]);
        assert_eq!(view.total, 2);
    }

    #[test]
    fn every_row_knows_its_explained_bytes() {
        let caches = category("demo.caches");
        let state = folders(&[
            ("/home", 100),
            ("/home/Caches", 40),
            ("/home/Documents", 50),
            ("/home/Documents/old.cache", 10),
            ("/home/Music", 10),
        ])
        .with_owner(Path::new("/home/Caches"), caches)
        .with_owner(Path::new("/home/Documents/old.cache"), caches);

        let view = open(&state, "/home");

        let explained: Vec<(String, Sizes)> =
            view.rows.iter().map(|row| (row.node.name.clone(), row.explained)).collect();
        assert_eq!(
            explained,
            [("Documents".to_owned(), sizes(10)), ("Caches".to_owned(), sizes(40)), ("Music".to_owned(), Sizes::ZERO)]
        );
    }

    #[test]
    fn rows_come_largest_first() {
        let state = state(&[
            ("/home", NodeKind::Folder, sizes(100)),
            ("/home/small", NodeKind::Folder, sizes(10)),
            ("/home/large.iso", NodeKind::File, sizes(60)),
            ("/home/middle", NodeKind::Folder, sizes(30)),
        ]);

        assert_eq!(row_names(&open(&state, "/home")), ["large.iso", "middle", "small"]);
    }

    #[test]
    fn a_partial_folder_is_partial() {
        let partial =
            Sizes { completeness: Completeness::Partial, ..Sizes::complete(ByteSize::new(40), ByteSize::new(40)) };
        let state = state(&[
            ("/home", NodeKind::Folder, Sizes { completeness: Completeness::Partial, ..sizes(100) }),
            ("/home/Library", NodeKind::Folder, partial),
            ("/home/Documents", NodeKind::Folder, sizes(60)),
        ]);

        let view = open(&state, "/home");

        let completeness: Vec<Completeness> = view.rows.iter().map(|row| row.node.sizes.completeness).collect();
        assert_eq!(completeness, [Completeness::Complete, Completeness::Partial]);
    }
}

mod property {
    use std::path::PathBuf;

    use fresher_core::{ByteSize, Completeness, Descent, DiskState, NodeKind, Sizes, StateId, Timestamp};
    use fresher_testing::FixedDiskState;
    use proptest::prelude::*;

    use super::category;
    use crate::settings::Ignored;
    use crate::storage::place::place_view;
    use crate::storage::shares::Shares;

    const MOST: usize = 20;
    const CATEGORIES: [&str; 2] = ["demo.caches", "demo.logs"];

    /// A random forest of `count` folders named `n<i>`: folder `i` hangs under `parents[i] % (i + 1)`, or is a root
    /// when that is `i`; its sizes are its own files' plus its children's.
    fn forest(count: usize, parents: &[usize], own: &[Sizes]) -> (Vec<PathBuf>, Vec<Sizes>) {
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
        let mut sizes: Vec<Sizes> = own.iter().take(count).copied().collect();
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

    fn own_sizes() -> impl Strategy<Value = Sizes> {
        (0_u64..1000, prop::bool::weighted(0.1)).prop_map(|(bytes, partial)| Sizes {
            completeness: if partial { Completeness::Partial } else { Completeness::Complete },
            ..Sizes::new(ByteSize::new(bytes), ByteSize::new(bytes * 2), ByteSize::new(bytes))
        })
    }

    proptest! {
        #[test]
        fn the_pages_of_a_folder_add_up_to_it_and_explain_no_more_than_each_row(
            count in 1_usize..=MOST,
            parents in prop::collection::vec(0_usize..1000, MOST),
            own in prop::collection::vec(own_sizes(), MOST),
            owners in prop::collection::vec(prop::option::weighted(0.3, 0_usize..2), MOST),
            limit in 1_u32..4,
        ) {
            let (paths, sizes) = forest(count, &parents, &own);
            let nodes = paths.iter().zip(&sizes).map(|(path, &sizes)| (path.clone(), NodeKind::Folder, sizes));
            let mut state = FixedDiskState::new(StateId::new(1), Timestamp::from_secs(0), nodes);
            for (path, owner) in paths.iter().zip(&owners) {
                if let Some(owner) = owner {
                    state = state.with_owner(path, category(CATEGORIES[*owner]));
                }
            }
            let shares = Shares::of(&state, &Ignored::default(), &crate::withheld::Withheld::offered());

            for path in &paths {
                let first = place_view(&state, &shares, state.find(path).unwrap(), Descent::Through, 0, limit).unwrap();
                let folder = first.path.last().unwrap().clone();
                let mut rows = first.rows;
                while u32::try_from(rows.len()).unwrap() < first.total {
                    let offset = u32::try_from(rows.len()).unwrap();
                    let next = place_view(&state, &shares, folder.id, Descent::Through, offset, limit).unwrap();
                    prop_assert_eq!(next.path.len(), 1, "a next page does not descend");
                    prop_assert!(!next.rows.is_empty());
                    rows.extend(next.rows);
                }
                let at = paths.iter().position(|path| state.find(path) == Some(folder.id)).unwrap();
                let total = rows.iter().fold(bytes(own[at]), |sum, row| {
                    let row = bytes(row.node.sizes);
                    [sum[0] + row[0], sum[1] + row[1], sum[2] + row[2]]
                });
                prop_assert_eq!(total, bytes(folder.sizes));
                for row in &rows {
                    let (explained, size) = (bytes(row.explained), bytes(row.node.sizes));
                    prop_assert!((0..3).all(|part| explained[part] <= size[part]), "{:?} > {:?}", explained, size);
                }
            }
        }
    }
}
