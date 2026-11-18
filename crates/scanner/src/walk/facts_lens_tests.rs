//! The facts lens of the test crate over the portable reader, through the full walk: a stated fact reads in the
//! match and in its folder's inside, and with no fact stated for a tree the lens is the reader.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::Path;
use std::sync::Arc;

use fresher_core::{Claim, EntryKind, FolderListing, FolderReader, KnownFolder, Match, Pattern, RawEntry, Timestamp};
use fresher_testing::{FIXTURE_NOW, FactsLens, FixtureTree};

use crate::matcher::testing::{interest, set_of};
use crate::walk::std_reader::StdReader;
use crate::walk::tests::{canonical, scan};

/// A home that states its facts: a folder `cache` 30 days old in full (a file and a socket inside), a folder `theirs`
/// holding a file of the system, and top-level `.log` files: one changed 30 days ago, a pipe, one of the system and
/// one that states nothing.
const STATED: &str = "[[dir]]\npath = \"cache\"\nage = \"30d\"\nchanged = \"30d\"\n\
    [[file]]\npath = \"cache/a\"\nage = \"30d\"\nchanged = \"30d\"\n\
    [[file]]\npath = \"cache/s\"\nkind = \"socket\"\nage = \"30d\"\nchanged = \"30d\"\n\
    [[file]]\npath = \"theirs/f\"\nowner = \"system\"\n\
    [[file]]\npath = \"old.log\"\nage = \"30d\"\nchanged = \"30d\"\naccessed = \"35d\"\n\
    [[file]]\npath = \"pipe.log\"\nkind = \"pipe\"\n\
    [[file]]\npath = \"root.log\"\nowner = \"system\"\nflags = [\"system-lock\"]\n\
    [[file]]\npath = \"new.log\"\n";

/// Scans the [`STATED`] home through the lens of its stated facts; the match at each path relative to the root.
fn scanned() -> BTreeMap<String, Match> {
    let tree = FixtureTree::from_toml_str(STATED).unwrap();
    let root = canonical(tree.root());
    let whole = |name: &str| interest(Pattern::Under(KnownFolder::Home, name.into()), Claim::Whole, &[]);
    let interests =
        set_of(&[whole("cache"), whole("theirs"), interest(Pattern::Extension("log".into()), Claim::Item, &[])]);
    let lens = FactsLens::new(StdReader::new(), tree.stated().clone());
    let scanned = scan(Arc::new(lens), &root, &interests, 2);
    let relative = |found: &Match| found.path.strip_prefix(&root).unwrap().to_string_lossy().into_owned();
    scanned.matches.into_iter().map(|found| (relative(&found), found)).collect()
}

#[expect(clippy::unnecessary_wraps, reason = "the times it is compared with are options")]
fn days_ago(days: u64) -> Option<Timestamp> {
    Some(Timestamp::from_secs(FIXTURE_NOW - days * 86_400))
}

/// The entries of one folder: its path, then each entry with its name, sorted by name.
type Listed = (String, Vec<(Vec<u8>, RawEntry)>);

/// Every entry of every folder below `root` (itself included), as `reader` lists them, by folder.
fn listings(reader: &dyn FolderReader, root: &Path) -> Vec<Listed> {
    let mut found = Vec::new();
    let mut folders = vec![root.to_path_buf()];
    let mut listing = FolderListing::new();
    while let Some(folder) = folders.pop() {
        reader.read(&folder, reader.identify(&folder).unwrap(), &mut listing).unwrap();
        let mut entries: Vec<(Vec<u8>, RawEntry)> =
            listing.iter().map(|(name, entry)| (name.to_vec(), *entry)).collect();
        entries.sort_by(|left, right| left.0.cmp(&right.0));
        for (name, entry) in &entries {
            if entry.kind == EntryKind::Folder {
                folders.push(folder.join(String::from_utf8_lossy(name).as_ref()));
            }
        }
        found.push((folder.display().to_string(), entries));
    }
    found
}

/// Sets every access time below `root` (itself included) to its modification time, so the first listing's reads do
/// not move one under the second listing's feet.
fn settle_tree(root: &Path) {
    fresher_testing::settle_access_time(root).unwrap();
    for entry in std::fs::read_dir(root).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            settle_tree(&path);
        } else {
            fresher_testing::settle_access_time(&path).unwrap();
        }
    }
}

mod negative {
    use std::path::PathBuf;

    use fresher_core::{EntryKind, FolderReader as _};
    use fresher_testing::{FactsLens, FixtureTree, Stated, StatedFacts, StatedOwner};
    use proptest::prelude::*;

    use super::{days_ago, listings, scanned, settle_tree, tree_text};
    use crate::walk::std_reader::StdReader;
    use crate::walk::tests::canonical;

    #[test]
    fn an_entry_not_stated_reads_as_the_reader_reads_it() {
        let matches = scanned();

        let new = &matches["new.log"];
        assert_eq!(new.kind, EntryKind::File);
        assert!(new.facts.owned_by_user);
        assert!(new.changed > days_ago(1), "a file written now changed now");
    }

    #[test]
    fn the_lens_never_answers_for_a_path_outside_the_tree() {
        let tree = FixtureTree::from_toml_str(super::STATED).unwrap();
        let other = FixtureTree::from_toml_str(super::STATED).unwrap();
        let root = canonical(other.root());
        settle_tree(&root);

        let lens = FactsLens::new(StdReader::new(), tree.stated().clone());

        assert_eq!(listings(&lens, &root), listings(&StdReader::new(), &root));
        assert_eq!(lens.access(&root.join("theirs")), StdReader::new().access(&root.join("theirs")));
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(24))]

        #[test]
        fn the_lens_with_no_fact_of_the_tree_is_the_reader(
            files in prop::collection::btree_set((0_u8..3, 0_u8..3, 0_u8..4), 1..12),
            empty in prop::collection::btree_set((0_u8..3, 0_u8..3), 0..4),
        ) {
            let tree = FixtureTree::from_toml_str(&tree_text(&files, &empty)).unwrap();
            let root = canonical(tree.root());
            settle_tree(&root);
            let mut elsewhere = StatedFacts::new();
            let theirs = Stated { owner: Some(StatedOwner::System), ..Stated::default() };
            elsewhere.insert(PathBuf::from("/nowhere/d0/f0"), theirs);

            for facts in [StatedFacts::new(), elsewhere] {
                let lens = FactsLens::new(StdReader::new(), facts);

                prop_assert_eq!(listings(&lens, &root), listings(&StdReader::new(), &root));
            }
        }
    }
}

mod positive {
    use fresher_core::EntryKind;

    use super::{days_ago, scanned};

    #[test]
    fn a_stated_change_reads_in_the_match_and_in_its_folders_inside() {
        let matches = scanned();

        let old = &matches["old.log"];
        assert_eq!((old.changed, old.accessed), (days_ago(30), days_ago(35)));
        let inside = matches["cache"].inside.unwrap();
        assert_eq!(inside.newest_change, days_ago(30));
        assert!(inside.complete && !inside.foreign);
    }

    #[test]
    fn a_stated_socket_or_pipe_is_another_kind() {
        let matches = scanned();

        assert_eq!(matches["pipe.log"].kind, EntryKind::Other);
        assert!(matches["cache"].inside.unwrap().special);
    }

    #[test]
    fn a_stated_owner_or_flag_is_the_systems() {
        let matches = scanned();

        let root = matches["root.log"].facts;
        assert!(!root.owned_by_user && root.system_immutable);
        assert!(matches["theirs"].inside.unwrap().foreign);
    }
}

/// A tree of files `d<a>/d<b>/f<c>` and empty folders `d<a>/e<b>`, as a description.
fn tree_text(files: &BTreeSet<(u8, u8, u8)>, empty: &BTreeSet<(u8, u8)>) -> String {
    let mut text = String::new();
    for (a, b, c) in files {
        let _ = writeln!(text, "[[file]]\npath = \"d{a}/d{b}/f{c}\"\nsize = {}", u32::from(*c) * 100);
    }
    for (a, b) in empty {
        let _ = writeln!(text, "[[dir]]\npath = \"d{a}/e{b}\"");
    }
    text
}
