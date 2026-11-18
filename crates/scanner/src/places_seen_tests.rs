use std::path::Path;

use fresher_core::{EntryKind, FolderIdentity, PlaceSeen, Unreadable};

use crate::matcher::trie::Segment;
use crate::places_seen::places_seen;
use crate::tree::arena::Tree;
use crate::tree::node::{Flag, NodeIndex};
use crate::tree::testing::rooted;

/// A home `/home` with `Library/Caches`: the tree, `Library` and `Caches`.
fn home() -> (Tree, NodeIndex, NodeIndex) {
    let (mut tree, root) = rooted("/home");
    let library = tree.add_child(root, b"Library", EntryKind::Folder).unwrap();
    let caches = tree.add_child(library, b"Caches", EntryKind::Folder).unwrap();
    (tree, library, caches)
}

/// What the frozen `tree` shows of the place of the interest anchored at `pattern` (an absolute path).
fn seen(mut tree: Tree, pattern: &str) -> Option<PlaceSeen> {
    tree.freeze();
    let anchors = [(0, Segment::parse(pattern.trim_start_matches('/')))];
    places_seen(&tree, &anchors).get(&0).copied()
}

mod negative {
    use super::*;

    #[test]
    fn the_place_unreadable_is_unread() {
        let (mut tree, _, caches) = home();
        tree.mark_unreadable(caches, Unreadable::Guarded).unwrap();

        assert_eq!(seen(tree, "/home/Library/Caches/*"), Some(PlaceSeen::Unread));
    }

    #[test]
    fn a_folder_on_the_way_unreadable_is_unread() {
        let (mut tree, root) = rooted("/home");
        let library = tree.add_child(root, b"Library", EntryKind::Folder).unwrap();
        tree.mark_unreadable(library, Unreadable::NoPermission).unwrap();

        assert_eq!(seen(tree, "/home/Library/Caches/*"), Some(PlaceSeen::Unread));
    }

    #[test]
    fn the_place_with_an_entry_not_read_is_unread() {
        let (mut tree, _, caches) = home();
        tree.mark(caches, Flag::Incomplete).unwrap();

        assert_eq!(seen(tree, "/home/Library/Caches/*"), Some(PlaceSeen::Unread));
    }

    #[test]
    fn a_folder_on_the_way_with_an_entry_not_read_is_unread() {
        let (mut tree, library, _) = home();
        tree.mark(library, Flag::Incomplete).unwrap();

        assert_eq!(seen(tree, "/home/Library/Caches/*"), Some(PlaceSeen::Unread));
    }

    #[test]
    fn the_place_not_entered_on_purpose_is_not_scanned() {
        for flags in [&[Flag::Avoided][..], &[Flag::OtherVolume], &[Flag::Avoided, Flag::OthersFolder]] {
            let (mut tree, _, caches) = home();
            for &flag in flags {
                tree.mark(caches, flag).unwrap();
            }

            assert_eq!(seen(tree, "/home/Library/Caches/*"), Some(PlaceSeen::NotScanned), "{flags:?}");
        }
    }

    #[test]
    fn a_folder_on_the_way_on_another_volume_is_not_scanned() {
        let (mut tree, root) = rooted("/home");
        let library = tree.add_child(root, b"Library", EntryKind::Folder).unwrap();
        tree.mark(library, Flag::OtherVolume).unwrap();

        assert_eq!(seen(tree, "/home/Library/Caches/*"), Some(PlaceSeen::NotScanned));
    }

    #[test]
    fn the_place_outside_every_root_is_not_scanned() {
        let (tree, _, _) = home();

        assert_eq!(seen(tree.clone(), "/elsewhere/Library/Caches/*"), Some(PlaceSeen::NotScanned));
        assert_eq!(seen(tree, "/*"), Some(PlaceSeen::NotScanned));
    }

    #[test]
    fn a_pattern_with_a_star_before_its_last_segment_has_no_place() {
        let (tree, _, _) = home();
        let shaped = vec![
            Segment::Literal(b"home".as_slice().into()),
            Segment::BundleId,
            Segment::Literal(b"Caches".as_slice().into()),
            Segment::Any,
        ];
        let mut frozen = tree.clone();
        frozen.freeze();

        assert_eq!(seen(tree, "/home/*/Caches/*"), None);
        assert_eq!(places_seen(&frozen, &[(0, shaped)]).get(&0), None);
    }

    #[test]
    fn a_collapsed_folder_on_the_way_is_unread() {
        let (mut tree, library, _) = home();
        tree.mark(library, Flag::Collapsed).unwrap();

        assert_eq!(seen(tree, "/home/Library/Caches/*"), Some(PlaceSeen::Unread));
    }

    #[test]
    fn a_folder_claimed_whole_on_the_way_is_unread() {
        let (mut tree, root) = rooted("/home");
        let library = tree.add_child(root, b"Library", EntryKind::Folder).unwrap();
        tree.mark(library, Flag::Opaque).unwrap();

        assert_eq!(seen(tree, "/home/Library/Caches/*"), Some(PlaceSeen::Unread));
    }

    #[test]
    fn a_place_claimed_whole_with_something_inside_not_read_is_unread() {
        let (mut tree, root) = rooted("/home");
        let library = tree.add_child(root, b"Library", EntryKind::Folder).unwrap();
        let caches = tree.add_child(library, b"Caches", EntryKind::Folder).unwrap();
        tree.mark(caches, Flag::Opaque).unwrap();
        tree.mark(caches, Flag::Incomplete).unwrap();

        assert_eq!(seen(tree, "/home/Library/Caches/*"), Some(PlaceSeen::Unread));
    }

    #[test]
    fn a_place_claimed_whole_by_another_interest_is_unread_though_read_in_full() {
        let (mut tree, root) = rooted("/home");
        let library = tree.add_child(root, b"Library", EntryKind::Folder).unwrap();
        let caches = tree.add_child(library, b"Caches", EntryKind::Folder).unwrap();
        tree.mark(caches, Flag::Opaque).unwrap();

        assert_eq!(seen(tree, "/home/Library/Caches/*"), Some(PlaceSeen::Unread));
    }
}

mod positive {
    use super::*;

    #[test]
    fn the_place_read_is_read_while_a_matched_folder_inside_is_unreadable() {
        let (mut tree, _, caches) = home();
        let safari = tree.add_child(caches, b"com.apple.Safari", EntryKind::Folder).unwrap();
        tree.mark_unreadable(safari, Unreadable::Guarded).unwrap();

        assert_eq!(seen(tree, "/home/Library/Caches/*"), Some(PlaceSeen::Read));
    }

    #[test]
    fn the_places_parent_read_without_it_is_absent() {
        let (mut tree, root) = rooted("/home");
        tree.add_child(root, b"Library", EntryKind::Folder).unwrap();

        assert_eq!(seen(tree.clone(), "/home/Library/Caches/*"), Some(PlaceSeen::Absent));
        assert_eq!(seen(tree, "/home/Movies/Caches/*"), Some(PlaceSeen::Absent));
    }

    #[test]
    fn the_place_a_file_or_a_link_is_absent() {
        for kind in [EntryKind::File, EntryKind::Symlink] {
            let (mut tree, root) = rooted("/home");
            let library = tree.add_child(root, b"Library", EntryKind::Folder).unwrap();
            tree.add_child(library, b"Caches", kind).unwrap();

            assert_eq!(seen(tree, "/home/Library/Caches/*"), Some(PlaceSeen::Absent), "{kind:?}");
        }
    }

    #[test]
    fn an_avoided_child_of_the_place_leaves_it_read() {
        let (mut tree, _, caches) = home();
        let book = tree.add_child(caches, b"AddressBook", EntryKind::Folder).unwrap();
        tree.mark(book, Flag::Avoided).unwrap();

        assert_eq!(seen(tree, "/home/Library/Caches/*"), Some(PlaceSeen::Read));
    }

    #[test]
    fn a_place_folded_for_the_cache_is_read() {
        let (mut tree, _, caches) = home();
        tree.mark(caches, Flag::Collapsed).unwrap();

        assert_eq!(seen(tree, "/home/Library/Caches/*"), Some(PlaceSeen::Read));
    }

    #[test]
    fn a_root_is_a_place_and_a_place_without_a_star_has_one() {
        let (tree, _, _) = home();

        assert_eq!(seen(tree.clone(), "/home/*"), Some(PlaceSeen::Read));
        assert_eq!(seen(tree, "/home/Library/Caches/JetBrains"), Some(PlaceSeen::Read));
    }

    #[test]
    fn each_interest_is_seen_under_its_own_root() {
        let (mut tree, _, caches) = home();
        tree.mark(caches, Flag::Incomplete).unwrap();
        let tmp = tree.add_root(Path::new("/tmp"), FolderIdentity { device: 1, file_id: 3 }).unwrap();
        tree.add_child(tmp, b"session", EntryKind::Folder).unwrap();
        tree.freeze();
        let anchors = [
            (2, Segment::parse("home/Library/Caches/*")),
            (5, Segment::parse("tmp/session/*")),
            (7, Segment::parse("tmp/gone/*")),
        ];

        let seen = places_seen(&tree, &anchors);

        let expected = [(2, PlaceSeen::Unread), (5, PlaceSeen::Read), (7, PlaceSeen::Absent)];
        assert_eq!(seen, expected.into());
    }
}
