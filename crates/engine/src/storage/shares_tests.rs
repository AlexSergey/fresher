use std::path::{Path, PathBuf};

use fresher_core::{ByteSize, CategoryId, DiskState, ModuleId, NodeId, NodeKind, Sizes, StateId, Timestamp};
use fresher_testing::FixedDiskState;

use crate::settings::Ignored;

fn category(id: &'static str) -> CategoryId {
    CategoryId::new(ModuleId::new("demo").unwrap(), id).unwrap()
}

fn sizes(bytes: u64) -> Sizes {
    Sizes::complete(ByteSize::new(bytes), ByteSize::new(bytes))
}

/// A state of folders with the sizes given, taken as they are.
fn state(nodes: &[(&str, Sizes)]) -> FixedDiskState {
    let nodes = nodes.iter().map(|&(path, sizes)| (PathBuf::from(path), NodeKind::Folder, sizes));
    FixedDiskState::new(StateId::new(1), Timestamp::from_secs(0), nodes)
}

fn ignoring(categories: &[&str], paths: &[&str]) -> Ignored {
    Ignored {
        categories: categories.iter().map(|&category| category.to_owned()).collect(),
        paths: paths.iter().map(PathBuf::from).collect(),
        ..Ignored::default()
    }
}

fn node(state: &FixedDiskState, path: &str) -> NodeId {
    state.find(Path::new(path)).unwrap()
}

mod negative {
    use std::path::Path;

    use fresher_core::{ByteSize, CategoryId, ModuleId, Sizes};

    use super::{category, ignoring, node, sizes, state};
    use crate::settings::Ignored;
    use crate::storage::shares::{RootShare, Shares};

    #[test]
    fn an_ignored_category_is_unclaimed() {
        let caches = category("demo.caches");
        let state =
            state(&[("/home", sizes(100)), ("/home/Caches", sizes(40))]).with_owner(Path::new("/home/Caches"), caches);

        let shares = Shares::of(&state, &ignoring(&["demo.caches"], &[]), &crate::withheld::Withheld::offered());

        assert_eq!(shares.category(caches), Sizes::ZERO);
        assert_eq!(shares.categories().count(), 0);
        assert_eq!(shares.explained(&state, node(&state, "/home/Caches")), Sizes::ZERO);
        assert_eq!(
            shares.roots(),
            [RootShare { root: node(&state, "/home"), explained: Sizes::ZERO, unclaimed: sizes(100) }]
        );
    }

    #[test]
    fn an_owner_of_a_hidden_module_is_unclaimed() {
        let caches = category("demo.caches");
        let other = CategoryId::new(ModuleId::new("other").unwrap(), "other.caches").unwrap();
        let state = state(&[("/home", sizes(100)), ("/home/Caches", sizes(40)), ("/home/Other", sizes(25))])
            .with_owner(Path::new("/home/Caches"), caches)
            .with_owner(Path::new("/home/Other"), other);
        let hidden = Ignored { hidden_modules: vec![ModuleId::new("demo").unwrap()], ..Ignored::default() };

        let shares = Shares::of(&state, &hidden, &crate::withheld::Withheld::offered());

        assert_eq!(shares.category(caches), Sizes::ZERO);
        assert_eq!(shares.category(other), sizes(25), "only the hidden module's owners leave");
        assert_eq!(shares.explained(&state, node(&state, "/home/Caches")), Sizes::ZERO);
        assert_eq!(shares.roots()[0].unclaimed, sizes(75));
    }

    #[test]
    fn an_owner_below_an_ignored_path_is_unclaimed() {
        let caches = category("demo.caches");
        let state = state(&[
            ("/home", sizes(100)),
            ("/home/Library", sizes(50)),
            ("/home/Library/Caches", sizes(40)),
            ("/home/LibraryX", sizes(30)),
        ])
        .with_owner(Path::new("/home/Library/Caches"), caches)
        .with_owner(Path::new("/home/LibraryX"), caches);

        let shares = Shares::of(&state, &ignoring(&[], &["/home/Library"]), &crate::withheld::Withheld::offered());

        assert_eq!(shares.category(caches), sizes(30), "a shared prefix is not below the ignored path");
        assert_eq!(shares.explained(&state, node(&state, "/home/Library")), Sizes::ZERO);
        assert_eq!(shares.roots()[0].explained, sizes(30));
        assert_eq!(shares.roots()[0].unclaimed, sizes(70));
    }

    #[test]
    fn a_nested_owner_is_not_counted_twice() {
        let (caches, logs) = (category("demo.caches"), category("demo.logs"));
        let state = state(&[("/home", sizes(100)), ("/home/Caches", sizes(60)), ("/home/Caches/app", sizes(25))])
            .with_owner(Path::new("/home/Caches"), caches)
            .with_owner(Path::new("/home/Caches/app"), logs);

        let shares = Shares::of(&state, &Ignored::default(), &crate::withheld::Withheld::offered());

        assert_eq!((shares.category(caches), shares.category(logs)), (sizes(35), sizes(25)));
        assert_eq!(shares.kept_by(node(&state, "/home/Caches")), Some((caches, sizes(35))));
        assert_eq!(shares.kept_by(node(&state, "/home/Caches/app")), Some((logs, sizes(25))));
        assert_eq!(shares.kept_by(node(&state, "/home")), None);
        assert_eq!(shares.roots()[0].explained, sizes(60));
        assert_eq!(shares.roots()[0].unclaimed, sizes(40));
    }

    #[test]
    fn an_ignored_owner_inside_a_counted_one_is_your_folder() {
        let (caches, logs) = (category("demo.caches"), category("demo.logs"));
        let state = state(&[("/home", sizes(100)), ("/home/Caches", sizes(60)), ("/home/Caches/app", sizes(25))])
            .with_owner(Path::new("/home/Caches"), caches)
            .with_owner(Path::new("/home/Caches/app"), logs);

        let shares = Shares::of(&state, &ignoring(&["demo.logs"], &[]), &crate::withheld::Withheld::offered());

        assert_eq!((shares.category(caches), shares.category(logs)), (sizes(35), Sizes::ZERO));
        assert_eq!(shares.kept_by(node(&state, "/home/Caches/app")), None, "an ignored owner keeps nothing");
        assert_eq!(shares.explained(&state, node(&state, "/home/Caches")), sizes(35));
        assert_eq!(shares.explained(&state, node(&state, "/home/Caches/app")), Sizes::ZERO);
        assert_eq!(shares.roots()[0].explained, sizes(35));
        assert_eq!(shares.roots()[0].unclaimed, sizes(65));
    }

    #[test]
    fn two_owners_side_by_side_do_not_share_bytes() {
        let (caches, logs) = (category("demo.caches"), category("demo.logs"));
        let state = state(&[("/home", sizes(100)), ("/home/a", sizes(30)), ("/home/b", sizes(20))])
            .with_owner(Path::new("/home/a"), caches)
            .with_owner(Path::new("/home/b"), logs);

        let shares = Shares::of(&state, &Ignored::default(), &crate::withheld::Withheld::offered());

        assert_eq!((shares.category(caches), shares.category(logs)), (sizes(30), sizes(20)));
        assert_eq!(shares.explained(&state, node(&state, "/home/a")), sizes(30));
        assert_eq!(shares.roots()[0].explained, sizes(50));
        assert_eq!(shares.roots()[0].unclaimed, sizes(50));
    }

    #[test]
    fn an_overflowing_sum_saturates() {
        let caches = category("demo.caches");
        let most = sizes(u64::MAX);
        let state = state(&[("/home", most), ("/home/a", most), ("/home/b", sizes(5))])
            .with_owner(Path::new("/home/a"), caches)
            .with_owner(Path::new("/home/b"), caches);

        let shares = Shares::of(&state, &Ignored::default(), &crate::withheld::Withheld::offered());

        assert_eq!(shares.category(caches), most);
        assert_eq!(shares.roots()[0].explained, most);
        assert_eq!(shares.roots()[0].unclaimed.allocated, ByteSize::ZERO);
    }
}

mod positive {
    use std::fs;
    use std::path::{Path, PathBuf};

    use fresher_core::{CancelToken, ClaimedPath, Completeness, DiskWalk, InterestSet, Sizes, WalkMode, WalkRequest};
    use fresher_testing::{EntryKind, FixtureTree, NaiveWalk, TreeReport};

    use super::{category, ignoring, node, sizes, state};
    use crate::settings::Ignored;
    use crate::storage::shares::Shares;

    /// The fixture `tree` walked by the reference walk and committed with every folder one level down claimed by
    /// `demo.outer` and every folder three levels down by `demo.inner`.
    fn walked(tree: &FixtureTree) -> std::sync::Arc<dyn fresher_core::DiskState> {
        let roots = [tree.root().to_path_buf()];
        let interests = InterestSet::new();
        let request =
            WalkRequest { roots: &roots, interests: &interests, avoid: &[], places: &[], mode: WalkMode::Full };
        let walk = NaiveWalk::new(tree.root());
        walk.walk(&request, &CancelToken::new(), &mut |_| {}, &mut |_| {}).unwrap();
        let claims: Vec<ClaimedPath> = TreeReport::of(tree.root())
            .unwrap()
            .entries
            .iter()
            .filter(|entry| entry.kind == EntryKind::Dir)
            .filter_map(|entry| {
                let owner = match entry.path.split('/').count() {
                    1 => category("demo.outer"),
                    3 => category("demo.inner"),
                    _ => return None,
                };
                Some(ClaimedPath { path: tree.root().join(&entry.path), category: owner })
            })
            .collect();
        walk.commit(&claims).unwrap()
    }

    fn fixtures() -> Vec<PathBuf> {
        let folder = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/homes");
        let mut fixtures: Vec<PathBuf> = fs::read_dir(folder)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.extension().is_some_and(|extension| extension == "toml"))
            .collect();
        fixtures.sort();
        fixtures
    }

    #[test]
    fn explained_plus_unclaimed_is_the_root() {
        let fixtures = fixtures();
        assert!(fixtures.len() > 1, "the homes are found");
        for fixture in fixtures {
            let tree = FixtureTree::from_toml(&fixture).unwrap();
            let state = walked(&tree);

            let shares = Shares::of(&*state, &Ignored::default(), &crate::withheld::Withheld::offered());

            assert_eq!(shares.roots().len(), 1, "{}", fixture.display());
            for share in shares.roots() {
                let root = state.node(share.root).unwrap().sizes;
                let whole = share.explained.checked_add(share.unclaimed).unwrap();
                let bytes = |sizes: Sizes| (sizes.logical, sizes.allocated, sizes.reclaimable);
                assert_eq!(bytes(whole), bytes(root), "{}", fixture.display());
            }
        }
    }

    #[test]
    fn a_folder_knows_its_explained_bytes() {
        let (caches, logs, other) = (category("demo.caches"), category("demo.logs"), category("demo.other"));
        let state = state(&[
            ("/home", sizes(1000)),
            ("/home/Documents", sizes(200)),
            ("/home/Library", sizes(600)),
            ("/home/Library/Caches", sizes(300)),
            ("/home/Library/Caches/app", sizes(50)),
            ("/home/Library/Logs", sizes(100)),
        ])
        .with_owner(Path::new("/home/Library/Caches"), caches)
        .with_owner(Path::new("/home/Library/Caches/app"), other)
        .with_owner(Path::new("/home/Library/Logs"), logs);

        let shares = Shares::of(&state, &Ignored::default(), &crate::withheld::Withheld::offered());

        let explained = |path: &str| shares.explained(&state, node(&state, path));
        assert_eq!(explained("/home"), sizes(400));
        assert_eq!(explained("/home/Library"), sizes(400));
        assert_eq!(explained("/home/Library/Caches"), sizes(300));
        assert_eq!(explained("/home/Library/Caches/app"), sizes(50));
        assert_eq!(explained("/home/Documents"), Sizes::ZERO);
        assert_eq!(shares.category(caches), sizes(250));
    }

    #[test]
    fn a_folder_inside_an_owner_is_wholly_explained() {
        let (caches, logs) = (category("demo.caches"), category("demo.logs"));
        let state = state(&[
            ("/home", sizes(1000)),
            ("/home/Caches", sizes(300)),
            ("/home/Caches/app", sizes(120)),
            ("/home/Caches/app/blobs", sizes(80)),
            ("/home/Caches/app/blobs/kept", sizes(30)),
            ("/home/Caches/app/blobs/hidden", sizes(20)),
        ])
        .with_owner(Path::new("/home/Caches"), caches)
        .with_owner(Path::new("/home/Caches/app/blobs/kept"), logs)
        .with_owner(Path::new("/home/Caches/app/blobs/hidden"), logs);

        let shares = Shares::of(
            &state,
            &ignoring(&[], &["/home/Caches/app/blobs/hidden"]),
            &crate::withheld::Withheld::offered(),
        );

        let explained = |path: &str| shares.explained(&state, node(&state, path));
        assert_eq!(explained("/home/Caches/app"), sizes(100), "all but the ignored owner inside");
        assert_eq!(explained("/home/Caches/app/blobs"), sizes(60));
        assert_eq!(explained("/home/Caches"), sizes(280));
        assert_eq!((shares.category(caches), shares.category(logs)), (sizes(250), sizes(30)));
    }

    #[test]
    fn partial_roots_give_partial_parts() {
        let caches = category("demo.caches");
        let partial = Sizes { completeness: Completeness::Partial, ..sizes(100) };
        let state = state(&[("/home", partial), ("/home/Caches", sizes(40)), ("/data", sizes(10))])
            .with_owner(Path::new("/home/Caches"), caches);

        let shares = Shares::of(&state, &Ignored::default(), &crate::withheld::Withheld::offered());

        let (home, data) = (shares.roots()[0], shares.roots()[1]);
        assert_eq!(shares.roots().len(), 2);
        assert_eq!(home.explained, Sizes { completeness: Completeness::Partial, ..sizes(40) });
        assert_eq!(home.unclaimed, Sizes { completeness: Completeness::Partial, ..sizes(60) });
        assert_eq!((data.explained, data.unclaimed), (Sizes::ZERO, sizes(10)));
    }
}

mod property {
    use std::path::PathBuf;

    use fresher_core::{ByteSize, Completeness, DiskState, NodeKind, Sizes, StateId, Timestamp};
    use fresher_testing::FixedDiskState;
    use proptest::prelude::*;

    use super::category;
    use crate::settings::Ignored;
    use crate::storage::shares::Shares;

    const MOST: usize = 24;
    const CATEGORIES: [&str; 3] = ["demo.caches", "demo.logs", "demo.data"];

    /// A random forest of `count` folders: folder `i` hangs under `parents[i] % (i + 1)`, or is a root when that is
    /// `i`; a folder's sizes are its own plus its children's, partial when any of them is.
    struct Forest {
        paths: Vec<PathBuf>,
        parents: Vec<Option<usize>>,
        /// The bytes of each folder alone, without its children.
        own: Vec<[u64; 3]>,
        sizes: Vec<Sizes>,
    }

    impl Forest {
        /// `index` and its ancestors, the nearest first.
        fn up(&self, index: usize) -> impl Iterator<Item = usize> + '_ {
            std::iter::successors(Some(index), |&at| self.parents[at])
        }
    }

    fn forest(count: usize, parents: &[usize], own: &[u64], partial: &[bool]) -> Forest {
        let mut paths: Vec<PathBuf> = Vec::with_capacity(count);
        let mut parent_of: Vec<Option<usize>> = Vec::with_capacity(count);
        for (index, &chosen) in parents.iter().take(count).enumerate() {
            let parent = chosen % (index + 1);
            let (path, parent) = if parent == index {
                (PathBuf::from(format!("/r{index}")), None)
            } else {
                (paths[parent].join(format!("n{index}")), Some(parent))
            };
            paths.push(path);
            parent_of.push(parent);
        }
        let mut sizes: Vec<Sizes> = (0..count)
            .map(|index| Sizes {
                completeness: if partial[index] { Completeness::Partial } else { Completeness::Complete },
                ..Sizes::new(ByteSize::new(own[index]), ByteSize::new(own[index] * 2), ByteSize::new(own[index]))
            })
            .collect();
        let own = sizes.iter().map(|&sizes| bytes(sizes)).collect();
        for index in (0..count).rev() {
            if let Some(parent) = parent_of[index] {
                sizes[parent] = sizes[parent].checked_add(sizes[index]).unwrap();
            }
        }
        Forest { paths, parents: parent_of, own, sizes }
    }

    fn bytes(sizes: Sizes) -> [u64; 3] {
        [sizes.logical.bytes(), sizes.allocated.bytes(), sizes.reclaimable.bytes()]
    }

    fn add(a: [u64; 3], b: [u64; 3]) -> [u64; 3] {
        [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
    }

    proptest! {
        #[test]
        fn the_shares_add_up_and_match_the_deepest_owner_of_every_byte(
            count in 1_usize..=MOST,
            parents in prop::collection::vec(0_usize..1000, MOST),
            own in prop::collection::vec(0_u64..1000, MOST),
            partial in prop::collection::vec(prop::bool::weighted(0.1), MOST),
            owners in prop::collection::vec(prop::option::weighted(0.4, 0_usize..3), MOST),
            ignored_categories in prop::collection::vec(prop::bool::weighted(0.3), 3),
            ignored_paths in prop::collection::vec(prop::bool::weighted(0.1), MOST),
        ) {
            let forest = forest(count, &parents, &own, &partial);
            let nodes = forest.paths.iter().zip(&forest.sizes).map(|(path, &sizes)| (path.clone(), NodeKind::Folder, sizes));
            let mut state = FixedDiskState::new(StateId::new(1), Timestamp::from_secs(0), nodes);
            for (path, owner) in forest.paths.iter().zip(&owners) {
                if let Some(owner) = owner {
                    state = state.with_owner(path, category(CATEGORIES[*owner]));
                }
            }
            let ignored = Ignored {
                categories: CATEGORIES.iter().zip(&ignored_categories).filter(|&(_, &on)| on).map(|(id, _)| (*id).to_owned()).collect(),
                paths: forest.paths.iter().zip(&ignored_paths).filter(|&(_, &on)| on).map(|(path, _)| path.clone()).collect(),
                ..Ignored::default()
            };

            let shares = Shares::of(&state, &ignored, &crate::withheld::Withheld::offered());

            let by_category = shares.categories().fold([0; 3], |total, (_, sizes)| add(total, bytes(sizes)));
            let by_root = shares.roots().iter().fold([0; 3], |total, share| add(total, bytes(share.explained)));
            prop_assert_eq!(by_category, by_root);
            for share in shares.roots() {
                let root = state.node(share.root).unwrap().sizes;
                prop_assert_eq!(add(bytes(share.explained), bytes(share.unclaimed)), bytes(root));
            }
            // The oracle: each folder's own bytes belong to the deepest owner at or above it.
            let keeper = |index: usize| forest.up(index).find_map(|at| owners[at].map(|owner| (at, owner)));
            let counted = |(at, owner): (usize, usize)| {
                !ignored_categories[owner] && !forest.up(at).any(|above| ignored_paths[above])
            };
            let mut oracle = [[0; 3]; 3];
            for index in 0..count {
                if let Some((_, owner)) = keeper(index).filter(|&keeper| counted(keeper)) {
                    oracle[owner] = add(oracle[owner], forest.own[index]);
                }
            }
            for (owner, id) in CATEGORIES.iter().enumerate() {
                prop_assert_eq!(bytes(shares.category(category(id))), oracle[owner], "{}", id);
            }
            for (index, path) in forest.paths.iter().enumerate() {
                let id = state.find(path).unwrap();
                let (explained, size) = (bytes(shares.explained(&state, id)), bytes(state.node(id).unwrap().sizes));
                prop_assert!(explained.iter().zip(size).all(|(&part, whole)| part <= whole), "{}", path.display());
                let expected = (0..count)
                    .filter(|&inner| forest.up(inner).any(|at| at == index))
                    .filter(|&inner| keeper(inner).is_some_and(counted))
                    .fold([0; 3], |total, inner| add(total, forest.own[inner]));
                prop_assert_eq!(explained, expected, "{}", path.display());
            }
        }
    }
}
