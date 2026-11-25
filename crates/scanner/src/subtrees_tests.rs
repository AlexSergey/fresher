use std::path::PathBuf;

use fresher_core::{ByteSize, EntryKind, Subtree, SubtreeQuery, Timestamp};

use crate::subtrees::subtrees;
use crate::tree::arena::Tree;
use crate::tree::node::NodeIndex;
use crate::tree::testing::{rooted, sums};

/// A project `/home/p` (time 100) holding `src` (200) and `node_modules` (900) with `node_modules/pkg` (950).
struct Project {
    tree: Tree,
    folder: NodeIndex,
    src: NodeIndex,
    modules: NodeIndex,
}

impl Project {
    fn new() -> Self {
        let (mut tree, root) = rooted("/home");
        let project = tree.add_child(root, b"p", EntryKind::Folder).unwrap();
        let src = tree.add_child(project, b"src", EntryKind::Folder).unwrap();
        let modules = tree.add_child(project, b"node_modules", EntryKind::Folder).unwrap();
        let pkg = tree.add_child(modules, b"pkg", EntryKind::Folder).unwrap();
        for (node, secs) in [(project, 100), (src, 200), (modules, 900), (pkg, 950)] {
            tree.set_modified(node, Some(Timestamp::from_secs(secs))).unwrap();
        }
        Self { tree, folder: project, src, modules }
    }

    /// The answer to one query, the tree frozen first.
    fn ask(mut self, folder: &str, skip: &[&str]) -> Option<Subtree> {
        self.tree.freeze();
        answer(&self.tree, folder, skip)
    }
}

fn query(folder: &str, skip: &[&str]) -> SubtreeQuery {
    SubtreeQuery { folder: PathBuf::from(folder), skip: skip.iter().map(PathBuf::from).collect() }
}

fn answer(tree: &Tree, folder: &str, skip: &[&str]) -> Option<Subtree> {
    subtrees(tree, &[query(folder, skip)]).remove(0)
}

#[expect(clippy::unnecessary_wraps, reason = "the times it is compared with are options")]
const fn at(secs: u64) -> Option<Timestamp> {
    Some(Timestamp::from_secs(secs))
}

mod negative {
    use std::sync::Arc;

    use fresher_core::{DiskWalk, Unreadable};
    use fresher_testing::FakeChangeJournal;

    use super::*;
    use crate::rescan::testing::Rig;
    use crate::tree::node::Flag;
    use crate::walk::std_reader::StdReader;

    #[test]
    fn a_path_the_tree_has_no_node_of_is_none() {
        for folder in ["/home/q", "/elsewhere", "/home/p/src/deeper"] {
            assert_eq!(Project::new().ask(folder, &[]), None, "{folder}");
        }
    }

    #[test]
    fn a_link_and_a_kept_file_are_none() {
        let mut project = Project::new();
        project.tree.add_child(project.folder, b"link", EntryKind::Symlink).unwrap();
        project.tree.add_child(project.folder, b"big.bin", EntryKind::File).unwrap();
        project.tree.freeze();

        assert_eq!(answer(&project.tree, "/home/p/link", &[]), None);
        assert_eq!(answer(&project.tree, "/home/p/big.bin", &[]), None);
    }

    #[test]
    fn a_folder_inside_a_folder_claimed_whole_is_none() {
        let mut project = Project::new();
        let whole = project.tree.add_child(project.folder, b"target", EntryKind::Folder).unwrap();
        project.tree.apply_opaque(whole, sums(3, 30, 4096, 0), 2, &[]).unwrap();
        project.tree.freeze();

        assert_eq!(answer(&project.tree, "/home/p/target/debug", &[]), None);
        assert!(answer(&project.tree, "/home/p/target", &[]).is_some());
    }

    #[test]
    fn a_folder_claimed_whole_below_it_makes_it_incomplete() {
        let mut project = Project::new();
        let whole = project.tree.add_child(project.src, b"build", EntryKind::Folder).unwrap();
        project.tree.apply_opaque(whole, sums(1, 10, 4096, 0), 3, &[]).unwrap();
        project.tree.set_modified(whole, at(700)).unwrap();

        let found = project.ask("/home/p", &["/home/p/node_modules"]).unwrap();

        assert!(!found.complete, "what changed inside it is not known");
        assert_eq!(found.newest, at(700));
    }

    #[test]
    fn a_skipped_folder_s_newer_time_does_not_count() {
        let found = Project::new().ask("/home/p", &["/home/p/node_modules"]).unwrap();

        assert_eq!(found.newest, at(200));
        assert!(found.complete);
    }

    #[test]
    fn an_unreadable_folder_makes_it_incomplete() {
        let mut project = Project::new();
        project.tree.mark_unreadable(project.src, Unreadable::NoPermission).unwrap();

        let found = project.ask("/home/p", &[]).unwrap();

        assert!(!found.complete);
        assert_eq!(found.newest, at(950), "its own time still counts");
    }

    #[test]
    fn an_avoided_or_other_volume_folder_makes_it_incomplete() {
        for flag in [Flag::Avoided, Flag::OtherVolume, Flag::Incomplete] {
            let mut project = Project::new();
            project.tree.mark(project.src, flag).unwrap();

            assert!(!project.ask("/home/p", &[]).unwrap().complete, "{flag:?}");
        }
    }

    #[test]
    fn a_collapsed_folder_makes_it_incomplete() {
        let (mut tree, root) = rooted("/home");
        let project = tree.add_child(root, b"p", EntryKind::Folder).unwrap();
        let folded = tree.add_child(project, b"docs", EntryKind::Folder).unwrap();
        tree.set_modified(folded, at(300)).unwrap();
        tree.freeze();
        tree.mark(folded, Flag::Collapsed).unwrap();

        let found = answer(&tree, "/home/p", &[]).unwrap();

        assert!(!found.complete);
        assert_eq!(found.newest, at(300));
    }

    #[test]
    fn an_unreadable_folder_that_is_skipped_keeps_it_complete() {
        let mut project = Project::new();
        let pkg = project.tree.children(project.modules).next().unwrap();
        project.tree.mark_unreadable(pkg, Unreadable::Guarded).unwrap();

        let found = project.ask("/home/p", &["/home/p/node_modules"]).unwrap();

        assert!(found.complete);
        assert_eq!(found.newest, at(200));
    }

    #[test]
    fn a_kept_file_s_time_does_not_count() {
        let mut project = Project::new();
        let file = project.tree.add_child(project.src, b"big.bin", EntryKind::File).unwrap();
        project.tree.set_modified(file, at(5_000)).unwrap();

        let found = project.ask("/home/p", &["/home/p/node_modules"]).unwrap();

        assert_eq!(found.newest, at(200));
    }

    #[test]
    fn before_any_walk_every_answer_is_none() {
        let rig = Rig::new("[[dir]]\npath = \"p\"\n", fresher_core::InterestSet::new());
        let scanner = rig.scanner(Arc::new(StdReader::new()), Arc::new(FakeChangeJournal::default()));

        let answers = scanner.subtrees(&[query(rig.path("p").to_str().unwrap(), &[]), query("/", &[])]);

        assert_eq!(answers, vec![None, None]);
    }
}

mod positive {
    use fresher_core::Completeness;

    use super::*;

    #[test]
    fn the_newest_time_is_the_newest_folder_below_it() {
        assert_eq!(Project::new().ask("/home/p", &[]).unwrap().newest, at(950));
        assert_eq!(Project::new().ask("/home/p/src", &[]).unwrap().newest, at(200));
    }

    #[test]
    fn a_folder_claimed_whole_counts_with_its_own_time() {
        let mut project = Project::new();
        let whole = project.tree.add_child(project.src, b"build", EntryKind::Folder).unwrap();
        project.tree.apply_opaque(whole, sums(1, 10, 4096, 0), 3, &[]).unwrap();
        project.tree.set_modified(whole, at(700)).unwrap();

        let found = project.ask("/home/p", &["/home/p/node_modules"]).unwrap();

        assert_eq!(found.newest, at(700));
    }

    #[test]
    fn a_skipped_folder_claimed_whole_keeps_it_complete() {
        let mut project = Project::new();
        let whole = project.tree.add_child(project.src, b"build", EntryKind::Folder).unwrap();
        project.tree.apply_opaque(whole, sums(1, 10, 4096, 0), 3, &[]).unwrap();

        let found = project.ask("/home/p", &["/home/p/node_modules", "/home/p/src/build"]).unwrap();

        assert!(found.complete);
        assert_eq!(found.newest, at(200));
    }

    #[test]
    fn the_sizes_are_the_frozen_subtree_s_sums() {
        let mut project = Project::new();
        project.tree.add_own(project.folder, sums(1, 1, 4096, 0)).unwrap();
        project.tree.add_own(project.src, sums(2, 10, 8192, 0)).unwrap();
        project.tree.add_own(project.modules, sums(3, 100, 12_288, 4096)).unwrap();

        let found = project.ask("/home/p", &["/home/p/node_modules"]).unwrap();

        assert_eq!(
            (found.sizes.logical, found.sizes.allocated, found.sizes.reclaimable),
            (ByteSize::new(111), ByteSize::new(24_576), ByteSize::new(4096)),
            "the skipped folder's sizes count"
        );
        assert_eq!(found.sizes.completeness, Completeness::Complete);
    }

    #[test]
    fn a_skip_that_is_no_node_or_lies_elsewhere_is_ignored() {
        let skips = ["/home/p/missing", "/home", "/home/p", "/elsewhere/node_modules", "/home/p/src/../node_modules"];
        for skip in skips {
            assert_eq!(Project::new().ask("/home/p", &[skip]).unwrap().newest, at(950), "{skip}");
        }
    }

    #[test]
    fn answers_come_in_the_order_of_the_queries() {
        let mut project = Project::new();
        project.tree.freeze();

        let answers = subtrees(
            &project.tree,
            &[query("/home/p/src", &[]), query("/home/q", &[]), query("/home/p/node_modules", &[])],
        );

        let newest: Vec<_> = answers.iter().map(|found| found.map(|found| found.newest)).collect();
        assert_eq!(newest, vec![Some(at(200)), None, Some(at(950))]);
    }
}
