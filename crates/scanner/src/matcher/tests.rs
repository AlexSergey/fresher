mod negative {
    use std::path::Path;

    use fresher_core::{Claim, EntryKind, KnownFolder, Pattern};
    use fresher_testing::{FixtureTree, NaiveWalk};

    use crate::matcher::Matcher;
    use crate::matcher::testing::{compile, interest, matched, reference, set_of};

    fn fixture(toml: &str) -> FixtureTree {
        FixtureTree::from_toml_str(toml).unwrap()
    }

    fn agrees_and_finds_nothing(tree: &FixtureTree, interests: &[fresher_core::Interest]) {
        let set = set_of(interests);
        assert_eq!(reference(tree, &set), []);
        assert_eq!(matched(tree, &set), []);
    }

    #[test]
    fn a_folder_name_does_not_match_a_file() {
        let modules = [interest(Pattern::FolderName("node_modules".into()), Claim::Whole, &[])];
        let matcher = compile(&set_of(&modules), Path::new("/home"));
        let state = matcher.root_state(Path::new("/home"));
        assert!(matcher.entry(state, b"node_modules", EntryKind::File).is_empty());
        agrees_and_finds_nothing(&fixture("[[file]]\npath = \"app/node_modules\"\n"), &modules);
    }

    #[test]
    fn an_extension_does_not_match_a_folder() {
        let logs = [interest(Pattern::Extension("log".into()), Claim::Item, &[])];
        let matcher = compile(&set_of(&logs), Path::new("/home"));
        let state = matcher.root_state(Path::new("/home"));
        assert!(matcher.entry(state, b"x.log", EntryKind::Folder).is_empty());
        assert!(matcher.entry(state, b".log", EntryKind::File).is_empty());
        assert!(matcher.entry(state, b"log", EntryKind::File).is_empty());
        agrees_and_finds_nothing(&fixture("[[dir]]\npath = \"x.log\"\n[[file]]\npath = \".log\"\n"), &logs);
    }

    #[test]
    fn a_look_alike_name_does_not_match() {
        let modules = [interest(Pattern::FolderName("node_modules".into()), Claim::Whole, &[])];
        let matcher = compile(&set_of(&modules), Path::new("/home"));
        let state = matcher.root_state(Path::new("/home"));
        for name in [&b"node_modules2"[..], b"Node_modules", b"node_module", b"xnode_modules"] {
            assert!(matcher.entry(state, name, EntryKind::Folder).is_empty(), "{name:?}");
        }
        let tree = fixture("[[dir]]\npath = \"node_modules2\"\n[[dir]]\npath = \"a/Node_modules\"\n");
        agrees_and_finds_nothing(&tree, &modules);
    }

    #[test]
    fn star_matches_exactly_one_name() {
        let caches = [interest(Pattern::Under(KnownFolder::Home, "Library/Caches/*".into()), Claim::Item, &[])];
        let tree = fixture("[[file]]\npath = \"Library/Caches/a/b\"\n");
        let set = set_of(&caches);
        let found = matched(&tree, &set);
        assert_eq!(found, reference(&tree, &set));
        assert_eq!(found.len(), 1);
        assert!(found[0].1.ends_with("Library/Caches/a"));
    }

    #[test]
    fn a_star_does_not_match_a_name_that_is_not_utf8() {
        let caches = [interest(Pattern::Under(KnownFolder::Home, "Caches/*".into()), Claim::Item, &[])];
        let matcher = compile(&set_of(&caches), Path::new("/home"));
        let caches = matcher.child_state(matcher.root_state(Path::new("/home")), b"Caches");
        assert!(matcher.entry(caches, b"\xff", EntryKind::Folder).is_empty());
        assert_eq!(matcher.entry(caches, b"ok", EntryKind::Folder).as_slice(), [0]);
    }

    #[test]
    fn an_anchor_that_does_not_resolve_matches_nothing() {
        let caches = [interest(Pattern::Under(KnownFolder::Home, "Library/Caches".into()), Claim::Item, &[])];
        let set = set_of(&caches);
        let tree = fixture("[[dir]]\npath = \"Library/Caches\"\n");
        let unresolved = Matcher::compile(&set, &|_| None);
        let relative = Matcher::compile(&set, &crate::matcher::compile::under_home(Path::new("relative/home")));
        for matcher in [unresolved, relative] {
            let state = matcher.root_state(tree.root());
            assert!(!state.is_live());
            assert!(matcher.entry(state, b"Library", EntryKind::Folder).is_empty());
        }
        let roots = [tree.root().to_path_buf()];
        let mut naive = Vec::new();
        let request = fresher_core::WalkRequest {
            roots: &roots,
            interests: &set,
            avoid: &[],
            places: &[],
            mode: fresher_core::WalkMode::Full,
        };
        fresher_core::DiskWalk::walk(
            &NaiveWalk::new("/no/such/home"),
            &request,
            &fresher_core::CancelToken::new(),
            &mut |found| naive.push(found),
            &mut |_| {},
        )
        .unwrap();
        assert_eq!(naive, []);
    }

    #[test]
    fn a_marker_that_is_a_folder_is_not_a_marker() {
        let tags = [interest(Pattern::MarkerFile("CACHEDIR.TAG".into()), Claim::Whole, &[])];
        let matcher = compile(&set_of(&tags), Path::new("/home"));
        let listing = crate::matcher::testing::listing(&[(b"CACHEDIR.TAG", EntryKind::Folder)]);
        assert_eq!(matcher.folder(&listing).markers(), &[] as &[u32]);
        let link = crate::matcher::testing::listing(&[(b"CACHEDIR.TAG", EntryKind::Symlink)]);
        assert_eq!(matcher.folder(&link).markers(), &[] as &[u32]);
        agrees_and_finds_nothing(&fixture("[[dir]]\npath = \"cache/CACHEDIR.TAG\"\n"), &tags);
    }

    #[test]
    fn a_pipe_named_like_a_marker_is_not_a_marker() {
        let tags = [interest(Pattern::MarkerFile("CACHEDIR.TAG".into()), Claim::Whole, &[])];
        let matcher = compile(&set_of(&tags), Path::new("/home"));
        let pipe = crate::matcher::testing::listing(&[(b"CACHEDIR.TAG", EntryKind::Other)]);
        assert_eq!(matcher.folder(&pipe).markers(), &[] as &[u32]);
        let tree = fixture("[[dir]]\npath = \"cache\"\n");
        fresher_testing::make_pipe(&tree.root().join("cache/CACHEDIR.TAG")).unwrap();
        agrees_and_finds_nothing(&tree, &tags);
    }

    #[test]
    fn an_unreadable_entry_is_no_marker_and_no_sibling() {
        let modules = [interest(Pattern::MarkerFile("Cargo.toml".into()), Claim::Item, &["Cargo.lock"])];
        let matcher = compile(&set_of(&modules), Path::new("/home"));
        let mut listing = fresher_core::FolderListing::new();
        let broken = fresher_core::RawEntry {
            error: Some(fresher_core::ReadError::Failed),
            ..crate::matcher::testing::raw(EntryKind::File)
        };
        listing.push(b"Cargo.toml", broken).unwrap();
        listing.push(b"Cargo.lock", broken).unwrap();
        let facts = matcher.folder(&listing);
        assert_eq!(facts.markers(), &[] as &[u32]);
        assert_eq!(facts.siblings_of(0), 0);
    }

    #[test]
    fn an_interest_with_more_than_64_siblings_matches_nothing() {
        let names: Vec<String> = (0..65).map(|index| format!("s{index}")).collect();
        let siblings: Vec<&str> = names.iter().map(String::as_str).collect();
        let wide = [interest(Pattern::FolderName("target".into()), Claim::Whole, &siblings)];
        let matcher = compile(&set_of(&wide), Path::new("/home"));
        let state = matcher.root_state(Path::new("/home"));
        assert!(matcher.entry(state, b"target", EntryKind::Folder).is_empty());
        assert!(!matcher.claims_whole(0));
        assert!(!matcher.claims_whole(7));
    }

    #[test]
    fn a_root_off_every_anchor_is_dead() {
        let caches = [interest(Pattern::Under(KnownFolder::Home, "Library/Caches".into()), Claim::Item, &[])];
        let matcher = compile(&set_of(&caches), Path::new("/home/user"));
        for root in ["/home/other", "/home/user/Documents", "relative", "/home/user/Library/../Library"] {
            assert!(!matcher.root_state(Path::new(root)).is_live(), "{root}");
        }
    }
}

mod positive {
    use std::path::Path;

    use fresher_core::{Claim, EntryKind, KnownFolder, Pattern};
    use fresher_testing::FixtureTree;

    use crate::matcher::testing::{compile, interest, listing, matched, reference, set_of};

    fn agrees(toml: &str, interests: &[fresher_core::Interest]) -> Vec<crate::matcher::testing::Found> {
        let tree = FixtureTree::from_toml_str(toml).unwrap();
        let set = set_of(interests);
        let found = matched(&tree, &set);
        assert_eq!(found, reference(&tree, &set));
        found
            .into_iter()
            .map(|(hit, path, siblings, marker)| {
                (hit, path.strip_prefix(tree.root()).unwrap().to_path_buf(), siblings, marker)
            })
            .collect()
    }

    #[test]
    fn a_folder_name_matches_at_any_depth() {
        let modules = [interest(Pattern::FolderName("node_modules".into()), Claim::Whole, &[])];
        let found = agrees("[[dir]]\npath = \"node_modules\"\n[[dir]]\npath = \"a/b/c/node_modules/x\"\n", &modules);
        let paths: Vec<_> = found.iter().map(|found| found.1.clone()).collect();
        assert_eq!(paths, [Path::new("a/b/c/node_modules"), Path::new("node_modules")]);
    }

    #[test]
    fn a_folder_name_matches_a_pipe_as_itself() {
        let tmp = [interest(Pattern::FolderName("tmp".into()), Claim::Item, &[])];
        let matcher = compile(&set_of(&tmp), Path::new("/home"));
        let state = matcher.root_state(Path::new("/home"));
        assert_eq!(matcher.entry(state, b"tmp", EntryKind::Other).as_slice(), [0]);
        assert_eq!(matcher.entry(state, b"tmp", EntryKind::File).as_slice(), &[] as &[u32]);

        let tree = FixtureTree::from_toml_str("[[dir]]\npath = \"a\"\n").unwrap();
        fresher_testing::make_pipe(&tree.root().join("a/tmp")).unwrap();
        let set = set_of(&tmp);
        let found = matched(&tree, &set);
        assert_eq!(found, reference(&tree, &set));
        assert_eq!(found.iter().map(|found| found.1.clone()).collect::<Vec<_>>(), [tree.root().join("a/tmp")]);
    }

    #[test]
    fn an_extension_matches_a_file_and_a_link() {
        let logs = [interest(Pattern::Extension("gz".into()), Claim::Item, &[])];
        let toml = "[[file]]\npath = \"b.tar.gz\"\n[[symlink]]\npath = \"l.gz\"\ntarget = \"/nowhere\"\n";
        let found = agrees(toml, &logs);
        assert_eq!(found.len(), 2);
        let matcher = compile(&set_of(&logs), Path::new("/home"));
        let state = matcher.root_state(Path::new("/home"));
        assert_eq!(matcher.entry(state, b"..gz", EntryKind::File).as_slice(), [0]);
    }

    #[test]
    fn an_anchored_path_matches_under_its_anchor_only() {
        let caches = [interest(Pattern::Under(KnownFolder::Home, "Library/Caches/*".into()), Claim::Item, &[])];
        let toml = "[[dir]]\npath = \"Library/Caches/app\"\n[[dir]]\npath = \"other/Library/Caches/app\"\n";
        let found = agrees(toml, &caches);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].1, Path::new("Library/Caches/app"));
    }

    #[test]
    fn a_root_below_an_anchor_starts_inside_the_trie() {
        let caches = [interest(Pattern::Under(KnownFolder::Home, "Library/Caches/*".into()), Claim::Item, &[])];
        let matcher = compile(&set_of(&caches), Path::new("/home/user"));
        let below = matcher.root_state(Path::new("/home/user/Library"));
        let inside = matcher.child_state(below, b"Caches");
        assert!(below.is_live() && inside.is_live());
        assert_eq!(matcher.entry(inside, b"app", EntryKind::File).as_slice(), [0]);
        let above = matcher.root_state(Path::new("/"));
        let home = matcher.child_state(matcher.child_state(above, b"home"), b"user");
        assert_eq!(home, matcher.root_state(Path::new("/home/user")));
        assert!(matcher.entry(home, b"Library", EntryKind::Folder).is_empty());
    }

    #[test]
    fn siblings_are_found_in_one_pass() {
        let interests = [
            interest(Pattern::FolderName("node_modules".into()), Claim::Whole, &["package.json", "yarn.lock"]),
            interest(Pattern::FolderName("target".into()), Claim::Whole, &["Cargo.toml"]),
            interest(Pattern::FolderName("build".into()), Claim::Whole, &["package.json"]),
        ];
        let matcher = compile(&set_of(&interests), Path::new("/home"));
        let facts = matcher.folder(&listing(&[
            (b"package.json", EntryKind::File),
            (b"node_modules", EntryKind::Folder),
            (b"yarn.lock", EntryKind::Symlink),
        ]));
        assert_eq!((facts.siblings_of(0), facts.siblings_of(1), facts.siblings_of(2)), (0b11, 0, 0b1));
        let toml =
            "[[file]]\npath = \"p/package.json\"\n[[dir]]\npath = \"p/node_modules\"\n[[dir]]\npath = \"p/build\"\n";
        let found = agrees(toml, &interests);
        assert_eq!(found[0].2, ["package.json"]);
    }

    #[test]
    fn two_interests_on_one_name_both_hit() {
        let interests = [
            interest(Pattern::FolderName("Caches".into()), Claim::Item, &[]),
            interest(Pattern::Under(KnownFolder::Home, "Library/Caches".into()), Claim::Item, &[]),
            interest(Pattern::FolderName("Caches".into()), Claim::Item, &[]),
        ];
        let matcher = compile(&set_of(&interests), Path::new("/home"));
        let library = matcher.child_state(matcher.root_state(Path::new("/home")), b"Library");
        assert_eq!(matcher.entry(library, b"Caches", EntryKind::Folder).as_slice(), [0, 1, 2]);
        assert_eq!(agrees("[[dir]]\npath = \"Library/Caches\"\n", &interests).len(), 3);
    }

    #[test]
    fn a_marker_file_matches_its_folder_and_a_tag_is_checked() {
        let interests = [
            interest(Pattern::MarkerFile("CACHEDIR.TAG".into()), Claim::Whole, &[]),
            interest(Pattern::MarkerFile("Cargo.toml".into()), Claim::Item, &[]),
        ];
        let matcher = compile(&set_of(&interests), Path::new("/home"));
        let facts = matcher.folder(&listing(&[(b"Cargo.toml", EntryKind::File), (b"CACHEDIR.TAG", EntryKind::File)]));
        assert_eq!(facts.markers(), [0, 1]);
        assert!(matcher.checks_cachedir_tag(0) && !matcher.checks_cachedir_tag(1));
        assert!(matcher.claims_whole(0) && !matcher.claims_whole(1));
        let toml = "[[file]]\npath = \"c/CACHEDIR.TAG\"\n[[file]]\npath = \"c/inner/Cargo.toml\"\n[[file]]\npath = \"r/Cargo.toml\"\n";
        let found = agrees(toml, &interests);
        let summary: Vec<_> = found.iter().map(|found| (found.0, found.1.clone(), found.3)).collect();
        assert_eq!(summary, [(0, Path::new("c").to_path_buf(), true), (1, Path::new("r").to_path_buf(), false)]);
    }
}

mod property {
    use std::collections::BTreeSet;
    use std::fmt::Write;
    use std::path::Path;

    use fresher_core::{Claim, EntryKind, Interest, InterestSet, KnownFolder, ModuleId, Pattern};
    use fresher_testing::FixtureTree;
    use proptest::prelude::*;

    use crate::matcher::testing::{compile, interest, listing, matched, reference, set_of};

    /// No two names differ only in case: the reference walk finds markers and siblings through the file system,
    /// which may ignore case, while the matcher compares names exactly.
    const NAMES: [&str; 11] = [
        "a",
        "node_modules",
        "node_modules2",
        "Library",
        "Caches",
        "x.log",
        ".log",
        "CACHEDIR.TAG",
        "package.json",
        "b.tar.gz",
        "gz",
    ];

    fn pattern() -> impl Strategy<Value = Pattern> {
        let name = || prop::sample::select(&NAMES[..]).prop_map(String::from);
        let segment = prop_oneof![3 => prop::sample::select(&NAMES[..]), 1 => Just("*")];
        prop_oneof![
            name().prop_map(Pattern::FolderName),
            name().prop_map(Pattern::MarkerFile),
            prop::sample::select(&["log", "gz", "json", "TAG"][..]).prop_map(|ext| Pattern::Extension(ext.into())),
            prop::collection::vec(segment, 1..4).prop_map(|parts| Pattern::Under(KnownFolder::Home, parts.join("/"))),
        ]
    }

    fn interests() -> impl Strategy<Value = Vec<Interest>> {
        let siblings = prop::collection::vec(prop::sample::select(&NAMES[..]), 0..3);
        prop::collection::vec((pattern(), any::<bool>(), siblings), 1..6).prop_map(|specs| {
            specs
                .into_iter()
                .map(|(pattern, whole, siblings)| {
                    interest(pattern, if whole { Claim::Whole } else { Claim::Item }, &siblings)
                })
                .collect()
        })
    }

    /// A fixture tree in TOML: entries under earlier folders, each a folder, a file or a link.
    fn tree() -> impl Strategy<Value = String> {
        prop::collection::vec((any::<usize>(), prop::sample::select(&NAMES[..]), 0..3_u8), 0..40).prop_map(|specs| {
            let mut folders = vec![String::new()];
            let mut taken = BTreeSet::new();
            let mut toml = String::new();
            for (parent, name, kind) in specs {
                let parent = &folders[parent % folders.len()];
                let path = if parent.is_empty() { name.to_owned() } else { format!("{parent}/{name}") };
                if !taken.insert(path.clone()) {
                    continue;
                }
                match kind {
                    0 => {
                        write!(toml, "[[dir]]\npath = \"{path}\"\n").unwrap();
                        folders.push(path);
                    }
                    1 => write!(toml, "[[file]]\npath = \"{path}\"\nsize = 10\n").unwrap(),
                    _ => write!(toml, "[[symlink]]\npath = \"{path}\"\ntarget = \"/nowhere\"\n").unwrap(),
                }
            }
            toml
        })
    }

    #[test]
    fn the_fingerprint_is_stable_across_runs() {
        let mut set = InterestSet::new();
        set.add(
            ModuleId::new("node-modules").unwrap(),
            &[interest(Pattern::FolderName("node_modules".into()), Claim::Whole, &["package.json"])],
        );
        set.add(
            ModuleId::new("demo").unwrap(),
            &[
                interest(Pattern::Under(KnownFolder::Home, "Library/Caches/*".into()), Claim::Item, &[]),
                interest(Pattern::MarkerFile("CACHEDIR.TAG".into()), Claim::Whole, &[]),
                interest(Pattern::Extension("log".into()), Claim::Item, &[]),
            ],
        );
        assert_eq!(compile(&set, Path::new("/Users/fixed")).fingerprint(), 0x3a31_54b2_3a10_b966);
    }

    #[test]
    fn the_fingerprint_changes_with_the_anchor() {
        let set = set_of(&[interest(Pattern::Under(KnownFolder::Home, "Library".into()), Claim::Item, &[])]);
        assert_ne!(
            compile(&set, Path::new("/Users/a")).fingerprint(),
            compile(&set, Path::new("/Users/b")).fingerprint()
        );
    }

    #[test]
    fn the_fingerprint_changes_with_the_held_names() {
        let anchored = || interest(Pattern::Under(KnownFolder::Home, ".npm/_cacache".into()), Claim::Whole, &[]);
        let held = |names: &[&str]| anchored().holding(names.iter().map(|&name| name.into()).collect()).unwrap();
        let home = Path::new("/Users/fixed");
        let print = |interest: Interest| compile(&set_of(&[interest]), home).fingerprint();

        assert_ne!(print(anchored()), print(held(&["index-v5"])));
        assert_ne!(print(held(&["index-v5"])), print(held(&["content-v2"])));
        assert_ne!(print(held(&["index-v5", "content-v2"])), print(held(&["content-v2", "index-v5"])));
        assert_eq!(print(held(&["index-v5"])), print(held(&["index-v5"])));
    }

    #[test]
    fn entry_counts_the_hits_of_a_64_interest_set() {
        let mut interests = Vec::new();
        for index in 0..16 {
            let name = format!("folder{index}");
            interests.push(interest(Pattern::FolderName(name.clone()), Claim::Whole, &["package.json"]));
            interests.push(interest(Pattern::Extension(format!("e{index}")), Claim::Item, &[]));
            interests.push(interest(Pattern::MarkerFile(format!("marker{index}")), Claim::Item, &[]));
            interests.push(interest(
                Pattern::Under(KnownFolder::Home, format!("Library/Caches{index}/*")),
                Claim::Item,
                &[],
            ));
        }
        let matcher = compile(&set_of(&interests), Path::new("/home"));
        let state = matcher.root_state(Path::new("/home"));
        let names: Vec<String> = (0..10_000).map(|index| format!("folder{}.e{}", index % 40, index % 20)).collect();
        let hits: usize = names
            .iter()
            .map(|name| {
                matcher.entry(state, name.as_bytes(), EntryKind::File).as_slice().len()
                    + matcher.entry(state, name.as_bytes(), EntryKind::Folder).as_slice().len()
            })
            .sum();
        // Files: names ending `.e0`–`.e15` hit one extension each (16 of every 20 names); folders: no exact name.
        assert_eq!(hits, 8_000);
        assert_eq!(matcher.folder(&listing(&[(b"marker3", EntryKind::File)])).markers(), [14]);
    }

    /// Run by hand: `cargo nextest run -p fresher-scanner --release --run-ignored only entry_takes`.
    #[test]
    #[ignore = "a timing, run by hand"]
    fn entry_takes_under_50_ns() {
        let mut interests = Vec::new();
        for index in 0..16 {
            interests.push(interest(Pattern::FolderName(format!("folder{index}")), Claim::Whole, &["package.json"]));
            interests.push(interest(Pattern::Extension(format!("e{index}")), Claim::Item, &[]));
            interests.push(interest(Pattern::MarkerFile(format!("marker{index}")), Claim::Item, &[]));
            interests.push(interest(
                Pattern::Under(KnownFolder::Home, format!("Library/Caches{index}/*")),
                Claim::Item,
                &[],
            ));
        }
        let matcher = compile(&set_of(&interests), Path::new("/home"));
        let state = matcher.root_state(Path::new("/home"));
        let names: Vec<String> = (0..100_000).map(|index| format!("some-file-{index}.e{}", index % 40)).collect();
        let started = std::time::Instant::now();
        let mut hits = 0;
        for _ in 0..10 {
            for name in &names {
                hits += matcher.entry(state, name.as_bytes(), EntryKind::File).as_slice().len();
            }
        }
        let per_entry = started.elapsed().as_nanos() / 1_000_000;
        eprintln!("entry: {per_entry} ns, {hits} hits");
        assert!(per_entry < 50, "{per_entry} ns per entry");
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]

        #[test]
        fn the_matcher_agrees_with_the_reference_walk(interests in interests(), toml in tree()) {
            let tree = FixtureTree::from_toml_str(&toml).unwrap();
            let set = set_of(&interests);
            prop_assert_eq!(matched(&tree, &set), reference(&tree, &set));
        }

        #[test]
        fn the_fingerprint_changes_with_any_interest(interests in interests(), change in 0..4_usize, which in any::<usize>()) {
            let before = compile(&set_of(&interests), Path::new("/home")).fingerprint();
            let mut changed = interests.clone();
            let position = which % changed.len();
            let old = &changed[position];
            let siblings: Vec<String> = old.siblings().to_vec();
            changed[position] = match change {
                0 => Interest::new(old.pattern().clone(), flip(old.claim()), siblings).unwrap(),
                1 => Interest::new(old.pattern().clone(), old.claim(), [siblings, vec!["extra".into()]].concat()).unwrap(),
                2 => Interest::new(renamed(old.pattern()), old.claim(), siblings).unwrap(),
                _ => {
                    changed.push(interest(Pattern::FolderName("more".into()), Claim::Item, &[]));
                    changed[position].clone()
                }
            };
            prop_assert_ne!(compile(&set_of(&changed), Path::new("/home")).fingerprint(), before);
        }
    }

    const fn flip(claim: Claim) -> Claim {
        match claim {
            Claim::Whole => Claim::Item,
            Claim::Item => Claim::Whole,
        }
    }

    fn renamed(pattern: &Pattern) -> Pattern {
        match pattern {
            Pattern::FolderName(name) => Pattern::MarkerFile(name.clone()),
            Pattern::MarkerFile(name) => Pattern::FolderName(name.clone()),
            Pattern::Extension(name) => Pattern::Extension(format!("{name}x")),
            // Another folder, the same path: the fingerprint names the folder.
            Pattern::Under(folder, path) => {
                let other = if *folder == KnownFolder::Home { KnownFolder::AppCaches } else { KnownFolder::Home };
                Pattern::Under(other, path.clone())
            }
        }
    }
}
