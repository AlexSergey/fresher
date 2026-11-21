//! Which installed app a folder of `Caches` is named after (design, section 2), on plain values. Never by name and
//! never the other way round: a vendor's folder `com.example` names no app, though apps' ids lie under it.

use std::ffi::OsStr;
use std::iter;

use fresher_core::{BundleId, InstalledApp};

/// The installed app a folder is named after.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Owner<'a> {
    pub(crate) app: &'a InstalledApp,
    /// The folder is named the app's bundle id itself; otherwise it is a part of the app (a helper's folder, or an
    /// id under the app's or a helper's).
    pub(crate) own: bool,
}

/// The bundle id a folder `name` is: valid UTF-8, a bundle id, with at least one dot.
pub(crate) fn id_of(name: &OsStr) -> Option<BundleId> {
    let name = name.to_str().filter(|name| name.contains('.'))?;
    BundleId::new(name).ok()
}

/// The app among `apps` whose bundle id or a helper's names `name` (`BundleId::names`): the longest naming id wins,
/// and among apps of the same naming id (two copies of one app), the first by path. `None` when no app names it.
pub(crate) fn owner<'a>(name: &str, apps: &'a [InstalledApp]) -> Option<Owner<'a>> {
    let naming = |app: &'a InstalledApp| {
        iter::once(&app.bundle_id).chain(&app.helpers).filter(|id| id.names(name)).map(|id| id.folded().len()).max()
    };
    let (_, app) = apps.iter().filter_map(|app| naming(app).map(|length| (length, app))).min_by(
        |(first, first_app), (second, second_app)| second.cmp(first).then_with(|| first_app.path.cmp(&second_app.path)),
    )?;
    Some(Owner { app, own: name.eq_ignore_ascii_case(app.bundle_id.as_str()) })
}

#[cfg(test)]
mod tests {
    use fresher_core::{BundleId, InstalledApp};

    /// An installed app of `id` at `/Applications/<name>.app`, with the helpers `helpers`.
    fn app(id: &str, name: &str, helpers: &[&str]) -> InstalledApp {
        let app = fresher_testing::installed_app(id, name, "1.0").unwrap();
        InstalledApp { helpers: helpers.iter().map(|helper| BundleId::new(*helper).unwrap()).collect(), ..app }
    }

    fn editor() -> InstalledApp {
        app("com.example.Editor", "Editor", &["com.example.Editor.Helper"])
    }

    mod negative {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt as _;

        use super::{app, editor};
        use crate::owner::{id_of, owner};

        #[test]
        fn a_name_without_a_dot_has_no_id() {
            for name in ["Editor", "JetBrains", "", ".DS_Store", "com.example.", "com..example", "com example.App"] {
                assert_eq!(id_of(OsStr::new(name)), None, "{name}");
            }
        }

        #[test]
        fn a_name_not_in_utf8_has_no_id() {
            assert_eq!(id_of(OsStr::from_bytes(b"com.example.\xff")), None);
        }

        #[test]
        fn a_folder_no_app_names_has_no_owner() {
            let apps = [editor()];

            for name in ["com.example.Viewer", "org.example.Editor", "com.example.Edito"] {
                assert_eq!(owner(name, &apps), None, "{name}");
            }
            assert_eq!(owner("com.example.Editor", &[]), None);
        }

        #[test]
        fn a_sibling_id_is_not_named() {
            assert_eq!(owner("com.example.Editorx", &[editor()]), None);
            let agent = app("com.example.Editor", "Editor", &["org.example.Agent"]);
            assert_eq!(owner("org.example.Agentx", &[agent]), None);
        }

        #[test]
        fn a_vendor_prefix_names_no_app() {
            assert_eq!(owner("com.example", &[editor()]), None);
            assert_eq!(owner("com", &[editor()]), None);
        }
    }

    mod positive {
        use std::ffi::OsStr;
        use std::path::PathBuf;

        use fresher_core::InstalledApp;

        use super::{app, editor};
        use crate::owner::{id_of, owner};

        #[test]
        fn an_id_shaped_name_has_its_id() {
            let id = id_of(OsStr::new("com.example.Editor")).unwrap();

            assert_eq!(id.as_str(), "com.example.Editor");
        }

        #[test]
        fn the_app_s_own_folder_is_own() {
            let apps = [editor()];

            let found = owner("com.example.Editor", &apps).unwrap();

            assert_eq!(found.app, &apps[0]);
            assert!(found.own);
        }

        #[test]
        fn a_helper_s_folder_is_a_part_of_its_app() {
            let apps = [editor()];

            let found = owner("com.example.Editor.Helper", &apps).unwrap();

            assert_eq!(found.app, &apps[0]);
            assert!(!found.own);
        }

        #[test]
        fn an_id_under_the_app_s_is_a_part() {
            let apps = [editor()];

            for name in ["com.example.Editor.Cache", "com.example.Editor.Helper.Cache"] {
                let found = owner(name, &apps).unwrap();

                assert_eq!(found.app, &apps[0], "{name}");
                assert!(!found.own, "{name}");
            }
        }

        #[test]
        fn the_longest_naming_id_wins() {
            let pro = app("com.example.Editor.Pro", "Editor Pro", &[]);
            let apps = [editor(), pro];

            for name in ["com.example.Editor.Pro", "com.example.Editor.Pro.Cache"] {
                assert_eq!(owner(name, &apps).unwrap().app, &apps[1], "{name}");
            }
            assert_eq!(owner("com.example.Editor.Cache", &apps).unwrap().app, &apps[0]);
            let reversed = [apps[1].clone(), apps[0].clone()];
            assert_eq!(owner("com.example.Editor.Pro", &reversed).unwrap().app, &reversed[0]);
        }

        #[test]
        fn two_copies_of_one_app_are_one_owner() {
            let copy = InstalledApp { path: PathBuf::from("/Applications/Old/Editor.app"), ..editor() };
            let apps = [copy, editor()];

            let found = owner("com.example.Editor", &apps).unwrap();

            assert_eq!(found.app.path, PathBuf::from("/Applications/Editor.app"));
            assert!(found.own);
        }

        #[test]
        fn case_is_ignored() {
            let apps = [editor()];

            let own = owner("COM.EXAMPLE.editor", &apps).unwrap();
            let part = owner("com.EXAMPLE.Editor.HELPER", &apps).unwrap();

            assert!(own.own);
            assert!(!part.own);
            assert_eq!(part.app, &apps[0]);
        }
    }
}
