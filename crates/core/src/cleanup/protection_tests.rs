use std::path::PathBuf;

use crate::cleanup::Protection;

fn paths(list: &[&str]) -> Vec<PathBuf> {
    list.iter().map(PathBuf::from).collect()
}

/// A small version of the macOS list: system places, the synced folders, the home and its own folders.
fn mac_like() -> Protection {
    Protection::new(
        paths(&["/System", "/usr", "/Users/ann/Library/Mobile Documents", "/Users/bob"]),
        paths(&["/Users", "/Users/ann", "/Users/ann/Library", "/Users/ann/Library/Caches"]),
        paths(&["/usr/local"]),
    )
    .unwrap()
}

/// A list with an app folder: `/Applications` opens its bundles; the app's own bundle is its own `never` place.
fn apps_like() -> Protection {
    Protection::new(
        paths(&["/System", "/Applications", "/Applications/Fresher.app", "/Users/bob"]),
        paths(&["/Users", "/Users/ann", "/Users/ann/Applications"]),
        vec![],
    )
    .unwrap()
    .with_app_folders(&paths(&["/Applications"]))
    .unwrap()
}

mod negative {
    use std::path::{Path, PathBuf};

    use super::{apps_like, mac_like, paths};
    use crate::cleanup::{Protected, Protection, ProtectionError};

    #[test]
    fn a_protected_path_is_refused() {
        assert_eq!(mac_like().check(Path::new("/System")), Err(Protected::Never));
        assert_eq!(mac_like().check(Path::new("/Users/bob")), Err(Protected::Never));
    }

    #[test]
    fn a_path_inside_a_protected_one_is_refused() {
        assert_eq!(mac_like().check(Path::new("/usr/bin/ls")), Err(Protected::Never));
        assert_eq!(
            mac_like().check(Path::new("/Users/ann/Library/Mobile Documents/com~apple~Notes")),
            Err(Protected::Never)
        );
    }

    #[test]
    fn an_ancestor_of_a_protected_path_is_refused() {
        for path in ["/", "/Users/ann/Library"] {
            assert!(mac_like().check(Path::new(path)).is_err(), "{path}");
        }
        let only_never = Protection::new(paths(&["/Users/ann/Library/Mobile Documents"]), vec![], vec![]).unwrap();
        for path in ["/", "/Users", "/Users/ann", "/Users/ann/Library"] {
            assert_eq!(only_never.check(Path::new(path)), Err(Protected::Ancestor), "{path}");
        }
    }

    #[test]
    fn a_whole_folder_is_refused_as_a_whole() {
        assert_eq!(mac_like().check(Path::new("/Users/ann/Library/Caches")), Err(Protected::WholeFolder));
        // The home is also an ancestor of a `never` place, which is checked first.
        assert_eq!(mac_like().check(Path::new("/Users/ann")), Err(Protected::Ancestor));
    }

    #[test]
    fn an_ancestor_of_a_whole_folder_is_refused() {
        let whole = Protection::new(vec![], paths(&["/Users/ann/Downloads"]), vec![]).unwrap();

        for path in ["/", "/Users", "/Users/ann"] {
            assert_eq!(whole.check(Path::new(path)), Err(Protected::WholeFolder), "{path}");
        }
    }

    #[test]
    fn a_carved_out_place_itself_is_refused() {
        assert_eq!(mac_like().check(Path::new("/usr/local")), Err(Protected::WholeFolder));
    }

    #[test]
    fn a_relative_or_dotted_path_is_not_accepted() {
        for path in ["usr", "/usr/./local", "/usr/local/..", "/a/../b", "./a"] {
            let error = Protection::new(paths(&[path]), vec![], vec![]);
            assert_eq!(error, Err(ProtectionError::NotPlain { path: path.into() }), "{path}");
            assert_eq!(Protection::default().check(Path::new(path)), Err(Protected::Never), "{path}");
            assert!(Protection::default().with_ignored(&paths(&[path])).is_err(), "{path}");
        }
    }

    #[test]
    fn an_except_outside_every_protected_place_is_refused() {
        let error = Protection::new(paths(&["/usr"]), vec![], paths(&["/opt/local"]));
        let itself = Protection::new(paths(&["/usr"]), vec![], paths(&["/usr"]));

        assert_eq!(error, Err(ProtectionError::LooseExcept { path: "/opt/local".into() }));
        assert_eq!(itself, Err(ProtectionError::LooseExcept { path: "/usr".into() }));
    }

    #[test]
    fn a_shared_prefix_is_not_inside() {
        let ann = Protection::new(paths(&["/Users/ann"]), vec![], vec![]).unwrap();

        assert_eq!(ann.check(Path::new("/Users/anna")), Ok(()));
        assert_eq!(ann.check(Path::new("/Users/anna/x")), Ok(()));
        assert_eq!(ann.check(Path::new("/Users/ann/x")), Err(Protected::Never));
    }

    #[test]
    fn an_ignored_path_and_its_ancestors_are_refused() {
        let ignoring = mac_like().with_ignored(&paths(&["/Users/ann/Library/Caches/keep"])).unwrap();

        assert_eq!(ignoring.check(Path::new("/Users/ann/Library/Caches/keep")), Err(Protected::Ignored));
        assert_eq!(ignoring.check(Path::new("/Users/ann/Library/Caches/keep/a")), Err(Protected::Ignored));
        assert_eq!(ignoring.check(Path::new("/Users/ann/Library/Caches/kept")), Ok(()));
        assert_eq!(mac_like().check(Path::new("/Users/ann/Library/Caches/keep")), Ok(()));
    }

    #[test]
    fn a_never_place_inside_a_carve_out_is_still_refused() {
        let nested = Protection::new(paths(&["/usr", "/usr/local/keep"]), vec![], paths(&["/usr/local"])).unwrap();

        assert_eq!(nested.check(Path::new("/usr/local/keep/x")), Err(Protected::Never));
        assert_eq!(nested.check(Path::new("/usr/local/other")), Ok(()));
    }

    #[test]
    fn a_carve_out_of_one_list_does_not_open_the_other() {
        let strict = Protection::new(paths(&["/usr"]), vec![], vec![]).unwrap();

        let joined = mac_like().joined(&strict).unwrap();

        assert_eq!(joined.check(Path::new("/usr/local/x")), Err(Protected::Never));
        assert_eq!(strict.joined(&mac_like()).unwrap().check(Path::new("/usr/local/x")), Err(Protected::Never));
        assert_eq!(joined.check(Path::new("/usr/local")), Err(Protected::Never));
    }

    #[test]
    fn a_path_inside_a_bundle_of_an_app_folder_is_refused() {
        for path in [
            "/Applications/A.app/Contents",
            "/Applications/A.app/Contents/MacOS/a",
            "/Applications/A.app/Contents/Info.plist",
            "/Applications/Utilities/A.app/Contents",
            "/Applications/Utilities/A.app/Contents/MacOS/a",
        ] {
            assert_eq!(apps_like().check(Path::new(path)), Err(Protected::Never), "{path}");
        }
    }

    #[test]
    fn the_app_folder_itself_and_a_folder_in_it_are_refused() {
        for path in ["/Applications", "/Applications/", "/Applications/Utilities", "/Applications/notes.txt"] {
            assert_eq!(apps_like().check(Path::new(path)), Err(Protected::Never), "{path}");
        }
    }

    #[test]
    fn a_bundle_two_folders_down_is_refused() {
        for path in ["/Applications/a/b/C.app", "/Applications/a/b/c/D.app", "/Applications/Utilities/x/A.app"] {
            assert_eq!(apps_like().check(Path::new(path)), Err(Protected::Never), "{path}");
        }
    }

    #[test]
    fn a_bundle_inside_a_bundle_is_refused() {
        for path in [
            "/Applications/A.app/B.app",
            "/Applications/A.APP/B.app",
            "/Applications/A.App/B.app",
            "/Applications/Utilities/A.app/B.app",
            "/Applications/A.app/Contents/Library/LoginItems/Helper.app",
        ] {
            assert_eq!(apps_like().check(Path::new(path)), Err(Protected::Never), "{path}");
        }
    }

    #[test]
    fn a_name_that_only_looks_like_a_bundle_is_refused() {
        for path in [
            "/Applications/A.app.zip",
            "/Applications/.app",
            "/Applications/A.APP",
            "/Applications/A.App",
            "/Applications/A.aPp",
            "/Applications/A.app ",
            "/Applications/A.appx",
            "/Applications/app",
            "/Applications/Aapp",
            "/Applications/Utilities/.app",
            "/Applications/.app/B.app",
            "/Applications/.APP/B.app",
            "/Applications/Utilities/A.APP",
            "/Applications/Utilities/A.app.zip",
        ] {
            assert_eq!(apps_like().check(Path::new(path)), Err(Protected::Never), "{path}");
        }
    }

    #[test]
    fn a_dotted_or_relative_path_into_an_app_folder_is_refused() {
        for path in [
            "/Applications/./A.app",
            "/Applications/x/../A.app",
            "/Applications/a/b/../C.app",
            "/Applications/A.app/..",
            "/Applications/A.app/.",
            "/Applications/../Applications/A.app",
            "Applications/A.app",
            "A.app",
        ] {
            assert_eq!(apps_like().check(Path::new(path)), Err(Protected::Never), "{path}");
        }
    }

    #[test]
    fn a_bundle_shaped_link_into_a_bundle_is_refused_by_its_physical_form() {
        // `/Applications/Link/B.app` through a link `Link` to `/Applications/A.app/Contents`: its physical form is
        // inside a bundle. (Its lexical form passes the protection; see the positive case.)
        assert_eq!(apps_like().check(Path::new("/Applications/A.app/Contents/B.app")), Err(Protected::Never));
        // Through a link named like a bundle, what is inside it is inside a bundle by name already.
        assert_eq!(apps_like().check(Path::new("/Applications/Link.app/Contents")), Err(Protected::Never));
    }

    #[test]
    fn a_case_variant_of_a_never_place_inside_an_app_folder_is_refused() {
        for path in ["/Applications/fresher.app", "/Applications/FRESHER.app", "/Applications/FreSher.app"] {
            assert_eq!(apps_like().check(Path::new(path)), Err(Protected::Never), "{path}");
        }
        let one_down = apps_like().with_refused(&paths(&["/Applications/Utilities/Kept.app"])).unwrap();
        for path in ["/Applications/utilities/kept.app", "/Applications/UTILITIES/Kept.app"] {
            assert_eq!(one_down.check(Path::new(path)), Err(Protected::Never), "{path}");
        }
        // A never place inside a bundle, named in another case, still holds the bundle back.
        let inside = apps_like().with_refused(&paths(&["/Applications/A.app/Contents/Kept"])).unwrap();
        assert_eq!(inside.check(Path::new("/Applications/a.app")), Err(Protected::Never));
    }

    #[test]
    fn a_case_variant_of_an_ignored_or_whole_place_inside_an_app_folder_is_refused() {
        let ignoring = apps_like().with_ignored(&paths(&["/Applications/Editor.app", "/Applications/Games"])).unwrap();
        for path in ["/Applications/editor.app", "/Applications/EDITOR.app", "/Applications/games/Chess.app"] {
            assert_eq!(ignoring.check(Path::new(path)), Err(Protected::Never), "{path}");
        }
        let whole = Protection::new(paths(&["/Applications"]), paths(&["/Applications/Tool.app"]), vec![])
            .unwrap()
            .with_app_folders(&paths(&["/Applications"]))
            .unwrap();
        assert_eq!(whole.check(Path::new("/Applications/tool.app")), Err(Protected::Never));
    }

    #[test]
    fn a_name_under_an_app_folder_that_is_not_utf8_is_refused() {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt as _;

        let bundle = Path::new("/Applications").join(OsStr::from_bytes(b"A\xff.app"));
        let one_down = Path::new("/Applications").join(OsStr::from_bytes(b"\xfe")).join("A.app");

        assert_eq!(apps_like().check(&bundle), Err(Protected::Never));
        assert_eq!(apps_like().check(&one_down), Err(Protected::Never));
    }

    #[test]
    fn a_never_place_inside_an_app_folder_stays_refused() {
        for path in ["/Applications/Fresher.app", "/Applications/Fresher.app/Contents/MacOS/fresher"] {
            assert_eq!(apps_like().check(Path::new(path)), Err(Protected::Never), "{path}");
        }
        let nested = apps_like().with_refused(&paths(&["/Applications/Utilities/Kept.app"])).unwrap();
        assert_eq!(nested.check(Path::new("/Applications/Utilities/Kept.app")), Err(Protected::Never));
        // A never place inside a bundle makes the bundle its ancestor.
        let inside = apps_like().with_refused(&paths(&["/Applications/A.app/Contents/Kept"])).unwrap();
        assert_eq!(inside.check(Path::new("/Applications/A.app")), Err(Protected::Ancestor));
    }

    #[test]
    fn an_ignored_bundle_is_refused() {
        let ignoring = apps_like()
            .with_ignored(&paths(&["/Applications/A.app", "/Applications/Games", "/Applications/B.app/Contents/x"]))
            .unwrap();

        for path in ["/Applications/A.app", "/Applications/Games/C.app", "/Applications/B.app"] {
            assert_eq!(ignoring.check(Path::new(path)), Err(Protected::Ignored), "{path}");
        }
        assert_eq!(ignoring.check(Path::new("/Applications/D.app")), Ok(()));
    }

    #[test]
    fn a_whole_bundle_in_an_app_folder_is_refused() {
        let whole = Protection::new(paths(&["/Applications"]), paths(&["/Applications/A.app"]), vec![])
            .unwrap()
            .with_app_folders(&paths(&["/Applications"]))
            .unwrap();

        assert_eq!(whole.check(Path::new("/Applications/A.app")), Err(Protected::WholeFolder));
    }

    #[test]
    fn app_folders_of_a_joined_list_are_refused() {
        let closed = Protection::new(paths(&["/Applications"]), vec![], vec![]).unwrap();
        let joined = closed.joined(&Protection::default()).unwrap();

        let opened = joined.with_app_folders(&paths(&["/Applications"]));

        assert_eq!(opened, Err(ProtectionError::AppFoldersOfAJoinedList));
        assert_eq!(joined.with_app_folders(&[]), Err(ProtectionError::AppFoldersOfAJoinedList));
        assert_eq!(joined.check(Path::new("/Applications/A.app")), Err(Protected::Never));
    }

    #[test]
    fn a_loose_app_folder_is_refused() {
        let loose = |folder: &str| apps_like().with_app_folders(&paths(&[folder]));
        for folder in ["/Library", "/Applications/Utilities", "/applications", "/APPLICATIONS", "/Users", "/"] {
            assert_eq!(loose(folder), Err(ProtectionError::LooseAppFolder { path: folder.into() }), "{folder}");
        }
        for folder in ["/Applications/../Applications", "/Applications/.", "Applications"] {
            assert_eq!(loose(folder), Err(ProtectionError::NotPlain { path: folder.into() }), "{folder}");
        }
        let carved = mac_like().with_app_folders(&paths(&["/usr/local"]));
        assert_eq!(carved, Err(ProtectionError::LooseAppFolder { path: "/usr/local".into() }));
        let empty = Protection::default().with_app_folders(&paths(&["/Applications"]));
        assert_eq!(empty, Err(ProtectionError::LooseAppFolder { path: "/Applications".into() }));
    }

    #[test]
    fn a_case_variant_of_an_app_folder_opens_nothing() {
        // The caller gives canonical paths: a case variant is another path, with the answer it had before.
        let closed = Protection::new(paths(&["/Applications"]), vec![], vec![]).unwrap();
        let open = closed.with_app_folders(&paths(&["/Applications"])).unwrap();
        for path in ["/applications/A.app", "/APPLICATIONS/A.app/Contents", "/Applications/A.APP"] {
            assert_eq!(open.check(Path::new(path)), closed.check(Path::new(path)), "{path}");
        }
    }

    #[test]
    fn a_list_without_app_folders_names_none() {
        let refused = apps_like().with_refused(&paths(&["/Volumes/Apps"])).unwrap();

        assert_eq!(mac_like().app_folders(), Vec::<PathBuf>::new());
        assert_eq!(Protection::default().app_folders(), Vec::<PathBuf>::new());
        assert_eq!(refused.app_folders(), paths(&["/Applications"]));
    }

    #[test]
    fn a_refused_place_is_never_and_its_ancestors_are_refused() {
        let refused = Protection::default().with_refused(&paths(&["/a/b"])).unwrap();

        for path in ["/a/b", "/a/b/c", "/a/b/c.app"] {
            assert_eq!(refused.check(Path::new(path)), Err(Protected::Never), "{path}");
        }
        for path in ["/", "/a"] {
            assert_eq!(refused.check(Path::new(path)), Err(Protected::Ancestor), "{path}");
        }
        assert_eq!(refused.check(Path::new("/a/bc")), Ok(()));
        // A carve-out of the list opens no refused place.
        let usr = mac_like().with_refused(&paths(&["/usr"])).unwrap();
        assert_eq!(usr.check(Path::new("/usr/local/x")), Err(Protected::Never));
        for path in ["usr", "/usr/../usr", "/usr/."] {
            let error = mac_like().with_refused(&paths(&[path]));
            assert_eq!(error, Err(ProtectionError::NotPlain { path: path.into() }), "{path}");
        }
    }

    #[test]
    fn a_refused_app_folder_lets_no_bundle_through() {
        let refused = apps_like().with_refused(&paths(&["/Applications"])).unwrap();
        for path in ["/Applications/A.app", "/Applications/Utilities/A.app", "/Applications/A.app/Contents"] {
            assert_eq!(refused.check(Path::new(path)), Err(Protected::Never), "{path}");
        }
        // Opening the folder again after the refusal opens only the list's own place, never the refused one.
        let reopened = refused.with_app_folders(&paths(&["/Applications"])).unwrap();
        assert_eq!(reopened.check(Path::new("/Applications/A.app")), Err(Protected::Never));
        let only_refused = Protection::default().with_refused(&paths(&["/Applications"])).unwrap();
        let opened = only_refused.with_app_folders(&paths(&["/Applications"]));
        assert_eq!(opened, Err(ProtectionError::LooseAppFolder { path: "/Applications".into() }));
    }

    #[test]
    fn another_lists_app_folder_opens_nothing_once_joined() {
        let strict = Protection::new(paths(&["/Applications"]), vec![], vec![]).unwrap();

        for joined in [apps_like().joined(&strict).unwrap(), strict.joined(&apps_like()).unwrap()] {
            for path in ["/Applications/A.app", "/Applications/Utilities/A.app"] {
                assert_eq!(joined.check(Path::new(path)), Err(Protected::Never), "{path}");
            }
        }
    }
}

mod positive {
    use std::path::Path;

    use super::{apps_like, mac_like, paths};
    use crate::cleanup::Protection;

    #[test]
    fn an_item_inside_a_whole_folder_may_go() {
        assert_eq!(mac_like().check(Path::new("/Users/ann/Library/Caches/x")), Ok(()));
        assert_eq!(mac_like().check(Path::new("/Users/ann/Downloads")), Ok(()));
    }

    #[test]
    fn an_item_inside_a_carve_out_may_go() {
        assert_eq!(mac_like().check(Path::new("/usr/local/x")), Ok(()));
        assert_eq!(mac_like().check(Path::new("/usr/local/x/y")), Ok(()));
    }

    #[test]
    fn an_unrelated_path_passes() {
        assert_eq!(mac_like().check(Path::new("/opt/homebrew/cache")), Ok(()));
    }

    #[test]
    fn a_bundle_at_the_top_of_an_app_folder_passes() {
        for path in ["/Applications/A.app", "/Applications/My App.app", "/Applications/A.b.app", "/Applications/A.app/"]
        {
            assert_eq!(apps_like().check(Path::new(path)), Ok(()), "{path}");
        }
    }

    #[test]
    fn a_bundle_one_folder_down_passes() {
        for path in ["/Applications/Utilities/A.app", "/Applications/My Tools/B.app", "/Applications/A.zip/C.app"] {
            assert_eq!(apps_like().check(Path::new(path)), Ok(()), "{path}");
        }
    }

    #[test]
    fn joined_lists_keep_the_opening_of_their_own_place() {
        let other = Protection::new(paths(&["/opt"]), vec![], vec![]).unwrap();

        for joined in [apps_like().joined(&other).unwrap(), other.joined(&apps_like()).unwrap()] {
            assert_eq!(joined.check(Path::new("/Applications/A.app")), Ok(()));
            assert_eq!(joined.check(Path::new("/Applications/Utilities/A.app")), Ok(()));
        }
    }

    #[test]
    fn a_refusal_of_another_place_keeps_the_bundles_open() {
        let refusing = apps_like().with_refused(&paths(&["/Volumes/Apps"])).unwrap();

        assert_eq!(refusing.check(Path::new("/Applications/A.app")), Ok(()));
    }

    #[test]
    fn a_bundle_shaped_path_through_a_link_passes_the_lexical_check() {
        // `/Applications/Link/B.app` with `Link` a link: the protection is lexical and lets it through; the cleaner's
        // open through no link (check 5, `O_NOFOLLOW_ANY`) refuses it, not this check.
        assert_eq!(apps_like().check(Path::new("/Applications/Link/B.app")), Ok(()));
    }

    #[test]
    fn a_bundle_whose_case_matches_no_protected_place_passes() {
        let ignoring = apps_like().with_ignored(&paths(&["/Applications/Editor.app"])).unwrap();

        assert_eq!(ignoring.check(Path::new("/Applications/Editors.app")), Ok(()));
        assert_eq!(ignoring.check(Path::new("/Applications/Fresher Helper.app")), Ok(()));
    }

    #[test]
    fn no_app_folder_changes_nothing() {
        assert_eq!(mac_like().with_app_folders(&[]).unwrap(), mac_like());
        assert_eq!(mac_like().with_refused(&[]).unwrap(), mac_like());
    }
}

mod property {
    use std::path::{Path, PathBuf};

    use proptest::prelude::*;

    use crate::cleanup::{Protected, Protection};

    /// A path of up to four names out of a few like names, so that equal, nested and shared-prefix paths come up.
    fn names() -> impl Strategy<Value = Vec<&'static str>> {
        prop::collection::vec(prop_oneof![Just("a"), Just("ab"), Just("b"), Just("a b")], 0..4)
    }

    fn path_of(names: &[&str]) -> PathBuf {
        std::iter::once("/").chain(names.iter().copied()).collect()
    }

    /// `inner` is `outer` or below it: the plain loop the model is written with.
    fn below(inner: &[&str], outer: &[&str]) -> bool {
        if inner.len() < outer.len() {
            return false;
        }
        for index in 0..outer.len() {
            if inner[index] != outer[index] {
                return false;
            }
        }
        true
    }

    /// The rules of `Protection::check`, written over lists of names.
    fn model(
        path: &[&str],
        never: &[Vec<&str>],
        whole: &[Vec<&str>],
        except: &[Vec<&str>],
        ignored: &[Vec<&str>],
    ) -> Result<(), Protected> {
        for place in never {
            let mut carved = false;
            for cut in except {
                if below(cut, place) && below(path, cut) {
                    carved = true;
                }
            }
            if below(path, place) && !carved {
                return Err(Protected::Never);
            }
        }
        for place in never {
            if below(place, path) && place.len() > path.len() {
                return Err(Protected::Ancestor);
            }
        }
        for place in whole.iter().chain(except) {
            if below(place, path) {
                return Err(Protected::WholeFolder);
            }
        }
        for place in ignored {
            if below(path, place) || below(place, path) {
                return Err(Protected::Ignored);
            }
        }
        Ok(())
    }

    /// Names that are, look like or hold app bundles.
    fn bundle_names(length: std::ops::Range<usize>) -> impl Strategy<Value = Vec<&'static str>> {
        prop::collection::vec(
            prop_oneof![Just("a"), Just("A.app"), Just("B.app"), Just("A.APP"), Just(".app"), Just("A.app.zip")],
            length,
        )
    }

    fn bundle_lists() -> impl Strategy<Value = Vec<Vec<&'static str>>> {
        prop::collection::vec(bundle_names(0..3), 0..3)
    }

    /// The two shapes an app folder lets through, written over names: `[<name>.app]` or `[<sub>, <name>.app]`.
    fn is_bundle_shape(names: &[&str]) -> bool {
        // As written: an `.APP` is no bundle name.
        let bundle = |name: &str| name.strip_suffix(".app").is_some_and(|stem| !stem.is_empty());
        let may_be_bundle = |name: &str| name.to_lowercase().ends_with(".app");
        match names {
            [name] => bundle(name),
            [sub, name] => !may_be_bundle(sub) && bundle(name),
            _ => false,
        }
    }

    fn lists() -> impl Strategy<Value = Vec<Vec<&'static str>>> {
        prop::collection::vec(names(), 0..3)
    }

    /// A random list: `never`, `whole`, carve-outs strictly inside the first `never` place, and ignored paths.
    fn protection() -> impl Strategy<Value = Protection> {
        (lists(), lists(), lists(), lists()).prop_map(|(never, whole, cuts, ignored)| {
            let to_paths = |list: &[Vec<&str>]| list.iter().map(|names| path_of(names)).collect::<Vec<_>>();
            let never: Vec<Vec<&str>> = never.into_iter().filter(|place| !place.is_empty()).collect();
            let except: Vec<PathBuf> = match never.first() {
                Some(place) => {
                    cuts.iter().filter_map(|cut| cut.first().map(|name| path_of(place).join(name))).collect()
                }
                None => Vec::new(),
            };
            Protection::new(to_paths(&never), to_paths(&whole), except)
                .unwrap()
                .with_ignored(&to_paths(&ignored))
                .unwrap()
        })
    }

    /// The order `check` answers in: the first answer is the gravest.
    fn gravity(answer: Result<(), Protected>) -> u8 {
        match answer {
            Err(Protected::Never) => 4,
            Err(Protected::Ancestor) => 3,
            Err(Protected::WholeFolder) => 2,
            Err(Protected::Ignored) => 1,
            Ok(()) => 0,
        }
    }

    proptest! {
        #[test]
        fn check_agrees_with_a_naive_component_model(
            path in names(), never in lists(), whole in lists(), cuts in lists(), ignored in lists(),
        ) {
            let never: Vec<Vec<&str>> = never.into_iter().filter(|place| !place.is_empty()).collect();
            // Only carve-outs strictly inside a `never` are accepted: each is the first one plus a name.
            let except: Vec<Vec<&str>> = match never.first() {
                Some(place) => cuts
                    .into_iter()
                    .filter_map(|cut| cut.first().map(|name| place.iter().copied().chain([*name]).collect()))
                    .collect(),
                None => Vec::new(),
            };
            let to_paths = |list: &[Vec<&str>]| list.iter().map(|names| path_of(names)).collect::<Vec<_>>();
            let protection = Protection::new(to_paths(&never), to_paths(&whole), to_paths(&except))
                .unwrap()
                .with_ignored(&to_paths(&ignored))
                .unwrap();

            let answer = protection.check(&path_of(&path));

            prop_assert_eq!(answer, model(&path, &never, &whole, &except, &ignored));
        }

        #[test]
        fn a_joined_list_refuses_what_either_refuses(
            path in names(), first in protection(), second in protection(),
        ) {
            let path = path_of(&path);

            let answer = first.joined(&second).unwrap().check(&path);

            let graver = [first.check(&path), second.check(&path)].into_iter().max_by_key(|answer| gravity(*answer));
            prop_assert_eq!(answer, graver.unwrap());
            if first.check(&path).is_err() || second.check(&path).is_err() {
                prop_assert!(answer.is_err());
            }
        }

        #[test]
        fn under_an_app_folder_only_the_two_bundle_shapes_pass(path in bundle_names(0..6)) {
            let folder = Protection::new(vec![PathBuf::from("/Applications")], vec![], vec![])
                .unwrap()
                .with_app_folders(&[PathBuf::from("/Applications")])
                .unwrap();
            let full: PathBuf = std::iter::once("/Applications").chain(path.iter().copied()).collect();

            let answer = folder.check(&full);

            prop_assert_eq!(answer.is_ok(), is_bundle_shape(&path), "{}", full.display());
            if answer.is_err() {
                prop_assert_eq!(answer, Err(Protected::Never));
            }
        }

        #[test]
        fn an_app_folder_changes_no_answer_but_a_bundles(
            path in bundle_names(0..5), never in bundle_lists(), whole in bundle_lists(), ignored in bundle_lists(),
            pick in 0..3_usize,
        ) {
            let to_paths = |list: &[Vec<&str>]| list.iter().map(|names| path_of(names)).collect::<Vec<_>>();
            let never: Vec<Vec<&str>> = never.into_iter().filter(|place| !place.is_empty()).collect();
            let closed = Protection::new(to_paths(&never), to_paths(&whole), vec![])
                .unwrap()
                .with_ignored(&to_paths(&ignored))
                .unwrap();
            let Some(folder) = never.get(pick % never.len().max(1)) else { return Ok(()) };
            let open = closed.with_app_folders(&[path_of(folder)]).unwrap();

            let (before, after) = (closed.check(&path_of(&path)), open.check(&path_of(&path)));

            let bundle = below(&path, folder) && is_bundle_shape(&path[folder.len()..]);
            if !bundle {
                prop_assert_eq!(after, before, "{}", path_of(&path).display());
            }
            if before.is_ok() {
                prop_assert_eq!(after, before);
            }
        }

        #[test]
        fn an_ancestor_of_anything_never_passes(never in lists(), whole in lists(), ignored in lists()) {
            let to_paths = |list: &[Vec<&str>]| list.iter().map(|names| path_of(names)).collect::<Vec<_>>();
            let protection = Protection::new(to_paths(&never), to_paths(&whole), vec![])
                .unwrap()
                .with_ignored(&to_paths(&ignored))
                .unwrap();

            for place in never.iter().chain(&whole).chain(&ignored) {
                for depth in 0..=place.len() {
                    let ancestor = path_of(&place[..depth]);
                    prop_assert!(protection.check(&ancestor).is_err(), "{}", ancestor.display());
                }
            }
            prop_assert!(protection.check(Path::new("relative")).is_err());
        }
    }
}
