//! What a matched folder holds: the names an anchored interest looks for in the folder's own listing, and whether a
//! held `CACHEDIR.TAG` is valid, through the full walk.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use fresher_core::{
    CACHEDIR_SIGNATURE, Claim, EntryAccess, FolderIdentity, FolderListing, FolderReader, Interest, KnownFolder, Match,
    Pattern, RawEntry, ReadError,
};
use fresher_testing::FixtureTree;

use crate::matcher::testing::{interest, set_of};
use crate::walk::std_reader::StdReader;
use crate::walk::tests::{Scanned, canonical, scan};

/// An interest in the home's `path`, claimed as `claim`, looking for `names` in it.
fn holding(path: &str, claim: Claim, names: &[&str]) -> Interest {
    interest(Pattern::Under(KnownFolder::Home, path.into()), claim, &[])
        .holding(names.iter().map(|&name| name.into()).collect())
        .unwrap()
}

/// The fixture home of `toml`, its `CACHEDIR.TAG` files written with `tags` (path, content).
fn home(toml: &str, tags: &[(&str, &[u8])]) -> (FixtureTree, PathBuf) {
    let tree = FixtureTree::from_toml_str(toml).unwrap();
    for (path, content) in tags {
        let path = tree.root().join(path);
        std::fs::write(&path, content).unwrap();
        fresher_testing::settle_access_time(&path).unwrap();
    }
    let root = canonical(tree.root());
    (tree, root)
}

fn walk(root: &Path, interests: &[Interest]) -> Scanned {
    walk_with(Arc::new(StdReader::new()), root, interests)
}

fn walk_with(reader: Arc<dyn FolderReader>, root: &Path, interests: &[Interest]) -> Scanned {
    let scanned = scan(reader, root, &set_of(interests), 2);
    scanned.summary.as_ref().unwrap();
    scanned
}

/// The match of interest `interest` at `path`.
fn found<'a>(scanned: &'a Scanned, path: &Path, interest: usize) -> &'a Match {
    scanned.matches.iter().find(|found| found.path == path && found.interest == interest).unwrap()
}

fn names(found: &Match) -> Vec<&str> {
    found.holds.iter().map(String::as_str).collect()
}

/// The portable reader, with the entry named `broken` read as failed.
struct Broken {
    inner: StdReader,
    broken: &'static str,
}

impl FolderReader for Broken {
    fn identify(&self, path: &Path) -> Result<FolderIdentity, ReadError> {
        self.inner.identify(path)
    }

    fn access(&self, path: &Path) -> Result<EntryAccess, ReadError> {
        self.inner.access(path)
    }

    fn read(&self, path: &Path, expected: FolderIdentity, out: &mut FolderListing) -> Result<(), ReadError> {
        self.inner.read(path, expected, out)?;
        let entries: Vec<(Vec<u8>, RawEntry)> = out.iter().map(|(name, entry)| (name.to_vec(), *entry)).collect();
        out.clear();
        for (name, entry) in entries {
            let error = (name == self.broken.as_bytes()).then_some(ReadError::Failed).or(entry.error);
            out.push(&name, RawEntry { error, ..entry })?;
        }
        Ok(())
    }

    fn head(&self, folder: &Path, name: &str, limit: usize) -> Result<Vec<u8>, ReadError> {
        self.inner.head(folder, name, limit)
    }
}

mod negative {
    use std::os::unix::fs::PermissionsExt as _;
    use std::sync::Arc;

    use fresher_core::{Claim, EntryKind, MarkerCheck};

    use super::{Broken, TagHome, found, holding, home, names, walk, walk_with};
    use crate::walk::std_reader::StdReader;

    #[test]
    fn a_folder_without_the_names_holds_none() {
        let (_tree, root) = home("[[file]]\npath = \".npm/_cacache/tmp/x\"\nsize = 1\n", &[]);

        let scanned = walk(&root, &[holding(".npm/_cacache", Claim::Whole, &["index-v5", "content-v2"])]);

        let cache = found(&scanned, &root.join(".npm/_cacache"), 0);
        assert_eq!(cache.holds, Vec::<String>::new());
        assert_eq!(cache.marker, None);
    }

    #[test]
    fn an_unreadable_entry_with_a_held_name_is_absent() {
        let (_tree, root) =
            home("[[dir]]\npath = \".npm/_cacache/index-v5\"\n[[dir]]\npath = \".npm/_cacache/content-v2\"\n", &[]);
        let reader = Arc::new(Broken { inner: StdReader::new(), broken: "index-v5" });

        let scanned = walk_with(reader, &root, &[holding(".npm/_cacache", Claim::Item, &["index-v5", "content-v2"])]);

        assert_eq!(names(found(&scanned, &root.join(".npm/_cacache"), 0)), ["content-v2"]);
    }

    #[test]
    fn a_folder_named_like_a_held_name_beside_the_match_is_not_held() {
        let (_tree, root) = home("[[dir]]\npath = \".npm/_cacache\"\n[[dir]]\npath = \".npm/index-v5\"\n", &[]);

        let scanned = walk(&root, &[holding(".npm/_cacache", Claim::Whole, &["index-v5"])]);

        assert_eq!(found(&scanned, &root.join(".npm/_cacache"), 0).holds, Vec::<String>::new());
    }

    #[test]
    fn a_tag_absent_a_folder_a_link_empty_or_without_the_signature_is_invalid() {
        let homes: [TagHome; 5] = [
            ("[[dir]]\npath = \"cache\"\n", &[], &[]),
            ("[[dir]]\npath = \"cache/CACHEDIR.TAG\"\n", &[], &[]),
            ("[[file]]\npath = \"tag\"\n[[symlink]]\npath = \"cache/CACHEDIR.TAG\"\ntarget = \"../tag\"\n", &[], &[]),
            ("[[dir]]\npath = \"cache\"\n", &[("cache/CACHEDIR.TAG", b"")], &["CACHEDIR.TAG"]),
            ("[[dir]]\npath = \"cache\"\n", &[("cache/CACHEDIR.TAG", b"Signature: 0")], &["CACHEDIR.TAG"]),
        ];
        for (toml, tags, held) in homes {
            let (tree, root) = home(toml, tags);
            if toml.contains("symlink") {
                std::fs::write(tree.root().join("tag"), fresher_core::CACHEDIR_SIGNATURE).unwrap();
            }

            let scanned = walk(&root, &[holding("cache", Claim::Item, &["CACHEDIR.TAG"])]);

            let cache = found(&scanned, &root.join("cache"), 0);
            assert_eq!((names(cache).as_slice(), cache.marker), (held, Some(MarkerCheck::Invalid)), "{toml}");
        }
    }

    #[test]
    fn a_held_name_on_a_file_match_gives_nothing() {
        let (_tree, root) = home("[[file]]\npath = \"cache\"\nsize = 3\n", &[]);

        let scanned = walk(&root, &[holding("cache", Claim::Item, &["CACHEDIR.TAG", "index-v5"])]);

        let cache = found(&scanned, &root.join("cache"), 0);
        assert_eq!((cache.kind, cache.holds.len(), cache.marker), (EntryKind::File, 0, None));
    }

    #[test]
    fn an_interest_without_names_gives_empty_holds_and_no_marker() {
        let (_tree, root) = home("[[dir]]\npath = \"cache/index-v5\"\n", &[("cache/CACHEDIR.TAG", super::SIGNED)]);
        let plain = crate::matcher::testing::interest(
            fresher_core::Pattern::Under(fresher_core::KnownFolder::Home, "cache".into()),
            Claim::Whole,
            &[],
        );

        let scanned = walk(&root, &[plain]);

        let cache = found(&scanned, &root.join("cache"), 0);
        assert_eq!((cache.holds.len(), cache.marker), (0, None));
    }

    #[test]
    fn a_folder_that_cannot_be_read_holds_nothing_and_has_no_marker_check() {
        for claim in [Claim::Item, Claim::Whole] {
            let (tree, root) = home("[[dir]]\npath = \"cache/index-v5\"\n", &[("cache/CACHEDIR.TAG", super::SIGNED)]);
            let cache = tree.root().join("cache");
            std::fs::set_permissions(&cache, std::fs::Permissions::from_mode(0o000)).unwrap();

            let scanned = walk(&root, &[holding("cache", claim, &["index-v5", "CACHEDIR.TAG"])]);
            std::fs::set_permissions(&cache, std::fs::Permissions::from_mode(0o755)).unwrap();

            let found = found(&scanned, &root.join("cache"), 0);
            assert_eq!((found.holds.len(), found.marker), (0, None), "{claim:?}");
        }
    }
}

/// A home's fixture, the tags written into it, and the names its `cache` holds.
type TagHome = (&'static str, &'static [(&'static str, &'static [u8])], &'static [&'static str]);

/// A valid tag's content.
const SIGNED: &[u8] = CACHEDIR_SIGNATURE;

mod positive {
    use fresher_core::{Claim, MarkerCheck, Pattern};

    use super::{SIGNED, found, holding, home, names, walk};
    use crate::matcher::testing::interest;

    #[test]
    fn a_whole_folder_holding_index_v5_and_content_v2_reports_both_in_the_interests_order() {
        let (_tree, root) = home(
            "[[file]]\npath = \".npm/_cacache/content-v2/sha512/a\"\nsize = 100\n\
             [[file]]\npath = \".npm/_cacache/index-v5/00/b\"\nsize = 10\n[[dir]]\npath = \".npm/_cacache/tmp\"\n",
            &[],
        );

        let scanned = walk(&root, &[holding(".npm/_cacache", Claim::Whole, &["index-v5", "content-v2"])]);

        let cache = found(&scanned, &root.join(".npm/_cacache"), 0);
        assert_eq!(names(cache), ["index-v5", "content-v2"]);
        assert_eq!((cache.marker, cache.sizes.logical.bytes()), (None, 110));
        assert_eq!(scanned.matches.len(), 1, "nothing inside a whole claim is matched");
    }

    #[test]
    fn an_item_folder_holding_a_valid_tag_is_valid_and_walked_on() {
        let (_tree, root) = home(
            "[[file]]\npath = \".cargo/registry/cache/a.json\"\nsize = 5\n",
            &[(".cargo/registry/CACHEDIR.TAG", SIGNED)],
        );
        let json = interest(Pattern::Extension("json".into()), Claim::Item, &[]);

        let scanned = walk(&root, &[holding(".cargo/registry", Claim::Item, &["CACHEDIR.TAG", "index"]), json]);

        let registry = found(&scanned, &root.join(".cargo/registry"), 0);
        assert_eq!((names(registry).as_slice(), registry.marker), (&["CACHEDIR.TAG"][..], Some(MarkerCheck::Valid)));
        assert!(scanned.matches.iter().any(|found| found.path == root.join(".cargo/registry/cache/a.json")));
    }

    #[test]
    fn two_interests_ending_at_the_same_folder_each_get_their_own_names() {
        let (_tree, root) =
            home("[[dir]]\npath = \"cache/a\"\n[[file]]\npath = \"cache/c\"\n", &[("cache/CACHEDIR.TAG", SIGNED)]);

        let scanned = walk(
            &root,
            &[holding("cache", Claim::Item, &["a", "CACHEDIR.TAG"]), holding("*", Claim::Whole, &["b", "c"])],
        );

        let item = found(&scanned, &root.join("cache"), 0);
        let whole = found(&scanned, &root.join("cache"), 1);
        assert_eq!((names(item).as_slice(), item.marker), (&["a", "CACHEDIR.TAG"][..], Some(MarkerCheck::Valid)));
        assert_eq!((names(whole).as_slice(), whole.marker), (&["c"][..], None));
    }

    #[test]
    fn the_last_of_64_names_is_held() {
        let (_tree, root) = home("[[file]]\npath = \"cache/name63\"\n", &[]);
        let names_64: Vec<String> = (0..64).map(|index| format!("name{index}")).collect();
        let held = interest(Pattern::Under(fresher_core::KnownFolder::Home, "cache".into()), Claim::Item, &[])
            .holding(names_64)
            .unwrap();

        let scanned = walk(&root, &[held]);

        assert_eq!(names(found(&scanned, &root.join("cache"), 0)), ["name63"]);
    }
}
