use fresher_core::{ByteSize, EntryAccess, EntryKind, FolderListing, RawEntry, Sizes};

use crate::matcher::entry::FolderFacts;
use crate::tree::arena::Tree;
use crate::tree::fold::{FolderBatch, KEEP_THRESHOLD, fold};
use crate::tree::node::NodeIndex;

const MIB: u64 = 1 << 20;

/// A readable entry of `kind` with `allocated` bytes, all private, and one link.
fn entry(kind: EntryKind, allocated: u64) -> RawEntry {
    RawEntry {
        kind,
        device: 1,
        file_id: 1,
        links: 1,
        sizes: Sizes::complete(ByteSize::new(allocated), ByteSize::new(allocated)),
        clone: None,
        modified: None,
        changed: None,
        accessed: None,
        cloud_only: false,
        access: EntryAccess::default(),
        error: None,
    }
}

/// A hard link of the file `file_id`, of `allocated` bytes.
fn hard_link(file_id: u64, allocated: u64) -> RawEntry {
    RawEntry { file_id, links: 2, ..entry(EntryKind::File, allocated) }
}

/// A clone of the group `clone`: `allocated` bytes, of which `private` are its own.
fn clone(clone: u64, allocated: u64, private: u64) -> RawEntry {
    let sizes = Sizes::new(ByteSize::new(allocated), ByteSize::new(allocated), ByteSize::new(private));
    RawEntry { clone: Some(clone), sizes, ..entry(EntryKind::File, allocated) }
}

fn listing(entries: &[(&str, RawEntry)]) -> FolderListing {
    let mut listing = FolderListing::new();
    for (name, entry) in entries {
        listing.push(name.as_bytes(), *entry).unwrap();
    }
    listing
}

fn batch(folder: NodeIndex, entries: &[(&str, RawEntry)]) -> FolderBatch {
    fold(folder, &listing(entries), &FolderFacts::default(), &[], KEEP_THRESHOLD, None)
}

/// Applies `batch` and returns the nodes it created.
fn apply(tree: &mut Tree, batch: &FolderBatch) -> Vec<NodeIndex> {
    let mut created = Vec::new();
    tree.apply(batch, &mut created).unwrap();
    created
}

mod negative {
    use fresher_core::{Completeness, EntryKind, RawEntry, ReadError};

    use super::{MIB, apply, batch, clone, entry, hard_link};
    use crate::tree::TreeError;
    use crate::tree::testing::{rooted, sums};

    #[test]
    fn a_small_file_is_not_kept() {
        let (_, root) = rooted("/home");
        let folded = batch(root, &[("small.txt", entry(EntryKind::File, MIB - 1))]);
        assert_eq!(folded.kept, []);
        assert_eq!(folded.folded, sums(1, MIB - 1, MIB - 1, MIB - 1));
    }

    #[test]
    fn a_hard_link_group_is_counted_once() {
        let (mut tree, root) = rooted("/home");
        let folders =
            apply(&mut tree, &batch(root, &[("a", entry(EntryKind::Folder, 0)), ("b", entry(EntryKind::Folder, 0))]));
        apply(&mut tree, &batch(folders[0], &[("x", hard_link(7, 4096)), ("y", hard_link(7, 4096))]));
        apply(&mut tree, &batch(folders[1], &[("z", hard_link(7, 4096))]));
        tree.freeze();
        assert_eq!(tree.sums(root), Some(sums(3, 4096, 4096, 4096)));
        assert_eq!(tree.sums(folders[0]), Some(sums(2, 4096, 4096, 4096)));
        assert_eq!(tree.sums(folders[1]), Some(sums(1, 0, 0, 0)));
    }

    #[test]
    fn a_clone_groups_shared_part_is_counted_once() {
        let (mut tree, root) = rooted("/home");
        let fully_shared = clone(3, 8192, 0);
        let diverged = clone(3, 8192, 1024);
        apply(&mut tree, &batch(root, &[("one", fully_shared), ("two", fully_shared), ("three", diverged)]));
        tree.freeze();
        // The shared part once (the most any member shares, 8192), the private parts per file (1024).
        assert_eq!(tree.sums(root), Some(sums(3, 3 * 8192, 9216, 1024)));
    }

    #[test]
    fn nothing_inside_an_opaque_folder_becomes_a_node() {
        let (mut tree, root) = rooted("/home");
        let created = apply(&mut tree, &batch(root, &[("node_modules", entry(EntryKind::Folder, 0))]));
        tree.apply_opaque(created[0], sums(10, 100, 200, 200), 4, &[]).unwrap();
        assert_eq!(tree.children(created[0]).count(), 0);
        assert_eq!(tree.len(), 2);
        assert_eq!(tree.apply_opaque(root, sums(1, 1, 1, 1), 0, &[]), Err(TreeError::NotOpaque));
    }

    #[test]
    fn a_folders_own_link_count_is_not_a_hard_link() {
        let (mut tree, root) = rooted("/home");
        let linked = RawEntry { links: 5, ..entry(EntryKind::Folder, 0) };
        let folded = batch(root, &[("a", linked), ("b", RawEntry { file_id: 2, ..linked })]);
        assert_eq!(folded.shared, []);
        let created = apply(&mut tree, &folded);
        tree.freeze();
        assert_eq!(created.len(), 2);
        assert_eq!(tree.node(root).unwrap().folders, 2);
    }

    #[test]
    fn an_unreadable_entry_counts_for_nothing_and_makes_the_folder_partial() {
        let (mut tree, root) = rooted("/home");
        let broken = RawEntry { error: Some(ReadError::Failed), ..entry(EntryKind::Folder, 0) };
        let folded = batch(root, &[("gone", broken), ("big", entry(EntryKind::File, 5 * MIB))]);
        assert!(folded.incomplete);
        assert_eq!(folded.kept.len(), 1);
        apply(&mut tree, &folded);
        tree.freeze();
        assert_eq!(tree.completeness(root), Some(Completeness::Partial));
        assert_eq!(tree.sums(root), Some(sums(1, 5 * MIB, 5 * MIB, 5 * MIB)));
    }

    #[test]
    fn a_link_nobody_matched_is_folded_with_no_size() {
        let (_, root) = rooted("/home");
        let folded = batch(root, &[("link", entry(EntryKind::Symlink, 4 * MIB))]);
        assert_eq!(folded.kept, []);
        assert_eq!(folded.folded, sums(1, 0, 0, 0));
    }
}

mod positive {
    use fresher_core::{Claim, EntryKind, Pattern, Timestamp};

    use super::{FolderBatch, MIB, apply, batch, entry, hard_link, listing};
    use crate::matcher::testing::{compile, interest, set_of};
    use crate::tree::fold::{KEEP_THRESHOLD, fold};
    use crate::tree::node::{Flag, Hit};
    use crate::tree::testing::{canonical, rooted, sums};

    #[test]
    fn a_large_file_is_kept_with_its_three_sizes() {
        let (mut tree, root) = rooted("/home");
        let big = fresher_core::RawEntry {
            sizes: fresher_core::Sizes::new(
                fresher_core::ByteSize::new(3 * MIB),
                fresher_core::ByteSize::new(2 * MIB),
                fresher_core::ByteSize::new(2 * MIB),
            ),
            modified: Some(Timestamp::from_secs(99)),
            ..entry(EntryKind::File, 2 * MIB)
        };
        let created = apply(&mut tree, &batch(root, &[("big.bin", big)]));
        tree.freeze();
        assert_eq!(tree.name(created[0]), b"big.bin");
        assert_eq!(tree.sums(created[0]), Some(sums(1, 3 * MIB, 2 * MIB, 2 * MIB)));
        assert_eq!(tree.node(created[0]).unwrap().modified(), Some(Timestamp::from_secs(99)));
        assert!(tree.node(created[0]).unwrap().flags.has(Flag::KeptFile));
    }

    #[test]
    fn a_matched_small_file_is_kept() {
        let interests = [interest(Pattern::Extension("log".into()), Claim::Item, &["package.json"])];
        let matcher = compile(&set_of(&interests), std::path::Path::new("/home"));
        let (mut tree, root) = rooted("/home");
        let entries = listing(&[("a.log", entry(EntryKind::File, 10)), ("package.json", entry(EntryKind::File, 10))]);
        let state = matcher.root_state(std::path::Path::new("/home"));
        let hits: Vec<_> = entries.iter().map(|(name, entry)| matcher.entry(state, name, entry.kind)).collect();
        let folded = fold(root, &entries, &matcher.folder(&entries), &hits, KEEP_THRESHOLD, None);
        assert_eq!(folded.kept.len(), 1);
        let created = apply(&mut tree, &folded);
        assert_eq!(tree.hits(created[0]), [Hit { interest: 0, marker: None, siblings: 0b1 }]);
        tree.freeze();
        assert_eq!(tree.sums(root), Some(sums(2, 20, 20, 20)));
    }

    #[test]
    fn link_bytes_go_to_the_first_path_in_name_order() {
        let build = |first: &str, second: &str| {
            let (mut tree, root) = rooted("/home");
            let folders = apply(
                &mut tree,
                &batch(root, &[(first, entry(EntryKind::Folder, 0)), (second, entry(EntryKind::Folder, 0))]),
            );
            for folder in folders {
                apply(&mut tree, &batch(folder, &[("big", hard_link(9, 2 * MIB))]));
            }
            tree.freeze();
            tree
        };
        for tree in [build("a", "b"), build("b", "a")] {
            let holder = tree.lookup(std::path::Path::new("/home/a/big")).unwrap();
            let copy = tree.lookup(std::path::Path::new("/home/b/big")).unwrap();
            assert_eq!(tree.sums(holder), Some(sums(1, 2 * MIB, 2 * MIB, 2 * MIB)));
            assert_eq!(tree.sums(copy), Some(sums(1, 0, 0, 0)));
            assert!(tree.node(copy).unwrap().flags.has(Flag::LinkCopy));
            assert!(!tree.node(holder).unwrap().flags.has(Flag::LinkCopy));
        }
        assert_eq!(canonical(&build("a", "b")), canonical(&build("b", "a")));
    }

    #[test]
    fn folded_sums_equal_the_files_they_replace() {
        let (_, root) = rooted("/home");
        let sizes = [1, 4096, 100_000, MIB - 1];
        let entries: Vec<(String, _)> =
            sizes.iter().map(|&size| (format!("f{size}"), entry(EntryKind::File, size))).collect();
        let named: Vec<(&str, _)> = entries.iter().map(|(name, entry)| (name.as_str(), *entry)).collect();
        let folded = batch(root, &named);
        let total: u64 = sizes.iter().sum();
        assert_eq!(folded.folded, sums(4, total, total, total));
    }

    #[test]
    fn an_opaque_folder_keeps_its_totals() {
        let (mut tree, root) = rooted("/home");
        let created = apply(&mut tree, &batch(root, &[("node_modules", entry(EntryKind::Folder, 0))]));
        let shared = [crate::tree::fold::SharedFile::of(&hard_link(4, 4096))];
        tree.apply_opaque(created[0], sums(10, 100, 200, 200), 4, &shared).unwrap();
        tree.freeze();
        assert!(tree.node(created[0]).unwrap().flags.has(Flag::Opaque));
        assert_eq!(tree.sums(created[0]), Some(sums(11, 4196, 4296, 4296)));
        assert_eq!(tree.node(created[0]).unwrap().folders, 4);
        assert_eq!(tree.node(root).unwrap().folders, 5);
    }

    #[test]
    fn apply_gives_the_kept_nodes_in_order_and_the_folders_own_hits() {
        let (mut tree, root) = rooted("/home");
        let mut folded = batch(root, &[("z", entry(EntryKind::Folder, 0)), ("a", entry(EntryKind::File, 2 * MIB))]);
        folded.own_hits.push(Hit { interest: 3, marker: Some(fresher_core::MarkerCheck::Valid), siblings: 0 });
        let created = apply(&mut tree, &folded);
        assert_eq!(created.iter().map(|&node| tree.name(node)).collect::<Vec<_>>(), [&b"z"[..], b"a"]);
        assert_eq!(tree.hits(root).len(), 1);
        let empty = FolderBatch::new(root);
        assert_eq!(empty.name(&folded.kept[0]), b"");
    }
}

mod property {
    use fresher_core::{ByteSize, EntryKind, RawEntry, Sizes};
    use proptest::prelude::*;

    use super::{apply, batch, entry, listing};
    use crate::matcher::entry::FolderFacts;
    use crate::tree::fold::fold;
    use crate::tree::node::NodeIndex;
    use crate::tree::testing::{canonical, rooted};

    /// A random entry: a folder, a link, or a file of up to 4 MiB that may be a hard link or a clone of a few groups.
    /// The members of one hard link group have the same sizes, as on a disk.
    fn raw() -> impl Strategy<Value = RawEntry> {
        (0..4_u8, 0..(4_u64 << 20), 0..4_u64, 0..4_u64, 0..4_u64).prop_map(
            |(kind, allocated, link, cloned, private)| match kind {
                0 => entry(EntryKind::Folder, 0),
                1 => entry(EntryKind::Symlink, 4096),
                _ if link > 0 => RawEntry { file_id: 100 + link, links: 2, ..entry(EntryKind::File, link * 300_000) },
                _ if cloned > 0 => {
                    let private = allocated / (private + 1);
                    let sizes = Sizes::new(ByteSize::new(allocated), ByteSize::new(allocated), ByteSize::new(private));
                    RawEntry { clone: Some(cloned), sizes, ..entry(EntryKind::File, allocated) }
                }
                _ => entry(EntryKind::File, allocated),
            },
        )
    }

    fn named(entries: &[RawEntry]) -> Vec<(String, RawEntry)> {
        entries.iter().enumerate().map(|(index, entry)| (format!("e{index}"), *entry)).collect()
    }

    proptest! {
        #[test]
        fn folding_never_changes_a_total(entries in prop::collection::vec(raw(), 0..30), low in 0..(4_u64 << 20), high in 0..(8_u64 << 20)) {
            let named = named(&entries);
            let listed = listing(&named.iter().map(|(name, entry)| (name.as_str(), *entry)).collect::<Vec<_>>());
            let total = |threshold| {
                let (mut tree, root) = rooted("/r");
                let folded = fold(root, &listed, &FolderFacts::default(), &[], threshold, None);
                apply(&mut tree, &folded);
                tree.freeze();
                tree.sums(root)
            };
            prop_assert_eq!(total(low), total(high));
            prop_assert_eq!(total(0), total(u64::MAX));
        }

        #[test]
        fn totals_do_not_depend_on_apply_order(folders in prop::collection::vec(prop::collection::vec(raw(), 0..8), 1..8)) {
            let build = |reversed: bool| {
                let (mut tree, root) = rooted("/r");
                let mut queue: Vec<(NodeIndex, usize)> = vec![(root, 0)];
                while !queue.is_empty() {
                    let (folder, spec) = if reversed { queue.remove(0) } else { queue.pop().unwrap() };
                    let named = named(folders.get(spec).map_or(&[][..], Vec::as_slice));
                    let entries: Vec<_> = named.iter().map(|(name, entry)| (name.as_str(), *entry)).collect();
                    let created = apply(&mut tree, &batch(folder, &entries));
                    let mut next = spec + 1;
                    for (node, (_, entry)) in created.iter().zip(entries.iter().filter(|(_, entry)| {
                        entry.kind == EntryKind::Folder || entry.sizes.allocated.bytes() >= super::KEEP_THRESHOLD
                    })) {
                        if entry.kind == EntryKind::Folder && next < folders.len() {
                            queue.push((*node, next));
                            next += 1;
                        }
                    }
                }
                tree.freeze();
                tree
            };
            prop_assert_eq!(canonical(&build(false)), canonical(&build(true)));
        }
    }
}
