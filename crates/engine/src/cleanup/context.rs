//! What the cleaner checks against, as it stands now: the scan roots in the settings, the places the last scan
//! walked outside them and the apps folders, the protection with the user's ignored paths (and, for every plan that is
//! not an uninstall, the apps folders refused), the access status probed again, and the guarded places under the
//! roots.

use std::path::{Path, PathBuf};

use fresher_core::{
    AccessStatus, BlockedReason, CleanupContext, CommandError, GuardedPlace, KnownFolder, KnownFolders, Protection,
};

use crate::engine::Shared;
use crate::error::lock;

/// The folders apps are installed in: an ordinary cleanup refuses them, only an uninstall may move a bundle there.
const APPS_FOLDERS: [KnownFolder; 2] = [KnownFolder::Applications, KnownFolder::UserApplications];

/// What a context is for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Scope {
    /// A plan of findings and picks: the apps folders are refused whole.
    Cleanup,
    /// An app's uninstall plan: the apps folders stay as the cleaner's own protection has them.
    Uninstall,
    /// Put back: it only brings back what the journal holds; nothing is refused besides the ignored paths.
    PutBack,
}

/// The owned parts of a [`CleanupContext`].
#[derive(Debug)]
pub(crate) struct Context {
    roots: Vec<PathBuf>,
    protection: Protection,
    access: AccessStatus,
    guarded: Vec<GuardedPlace>,
}

impl Context {
    /// The context of `scope` now; the access is probed again first (`AccessChanged` when it changed). The roots hold
    /// each located apps folder no root holds. The protection refuses the user's ignored paths, what is inside them and
    /// what holds them, and for `Scope::Cleanup` every located apps folder, every app folder of the cleaner's own
    /// protection (located or not) and what is inside them; the cleaner keeps the protected places it was built with.
    /// A protection that cannot be made blocks the cleanup: it fails closed.
    pub(crate) fn now(shared: &Shared, scope: Scope) -> Result<Self, CommandError> {
        let (mut roots, ignored) = {
            let settings = lock(&shared.settings)?;
            (settings.scan_roots(), settings.ignored_paths())
        };
        roots.extend(lock(&shared.results)?.walked().iter().map(|place| place.path.clone()));
        let apps = apps_folders(&*shared.ports.system.known_folders);
        let home = shared.home();
        let unheld: Vec<PathBuf> =
            apps.iter().filter(|folder| !holds(&roots, folder, home.as_deref())).cloned().collect();
        roots.extend(unheld);
        let blocked = |error: fresher_core::ProtectionError| {
            tracing::warn!(%error, "the cleanup's protection cannot be made");
            CommandError::CleaningBlocked { reason: BlockedReason::Settings }
        };
        let mut protection = Protection::default().with_ignored(&ignored).map_err(blocked)?;
        if scope == Scope::Cleanup {
            let cleaners = shared.cleanup.as_ref().map(|cleanup| cleanup.cleaner.protection().app_folders());
            let refused: Vec<PathBuf> = apps.iter().cloned().chain(cleaners.into_iter().flatten()).collect();
            protection = protection.with_refused(&refused).map_err(blocked)?;
        }
        shared.refresh_access();
        let access = shared.access.status();
        let guarded = shared.ports.access.guarded_places(&roots);
        Ok(Self { roots, protection, access, guarded })
    }

    pub(crate) fn borrow(&self) -> CleanupContext<'_> {
        CleanupContext {
            roots: &self.roots,
            protection: &self.protection,
            access: &self.access,
            guarded: &self.guarded,
        }
    }
}

/// The apps folders the system locates, in [`APPS_FOLDERS`]' order.
pub(crate) fn apps_folders(folders: &dyn KnownFolders) -> Vec<PathBuf> {
    APPS_FOLDERS.into_iter().filter_map(|folder| folders.locate(folder)).collect()
}

/// Whether one of `roots` (`~` read as `home`) holds `path` or is it, by components.
pub(crate) fn holds(roots: &[PathBuf], path: &Path, home: Option<&Path>) -> bool {
    roots.iter().any(|root| {
        let expanded = match (root.strip_prefix("~"), home) {
            (Ok(rest), Some(home)) if rest.as_os_str().is_empty() => home.to_path_buf(),
            (Ok(rest), Some(home)) => home.join(rest),
            _ => root.clone(),
        };
        path.starts_with(expanded)
    })
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use fresher_core::Protected;

    use super::{Context, Scope};
    use crate::Engine;
    use crate::apps::fixture::{app, facts, fixture, state};
    use crate::testing::scan;

    /// An engine after one scan of the apps tests' home, with one app installed: `/Applications` and
    /// `/home/Applications` are its apps folders.
    fn engine() -> Engine {
        let editor = app("com.example.Editor", "Editor");
        let apps =
            fresher_testing::FakeInstalledApps::new(vec![editor.clone()]).with_bundle(&editor.path, Ok(facts(1, None)));
        let engine = fixture(
            vec![state(1, &[("/home", fresher_core::NodeKind::Folder, 0)])],
            std::sync::Arc::new(apps),
            crate::apps::fixture::idle(),
        )
        .engine();
        scan(&engine);
        engine
    }

    fn check(cx: &Context, path: &str) -> Result<(), Protected> {
        cx.borrow().protection.check(Path::new(path))
    }

    mod negative {
        use std::path::PathBuf;

        use fresher_core::Protected;

        use super::{Context, Scope, check, engine};

        #[test]
        fn an_ordinary_cleanup_refuses_the_apps_folder() {
            let engine = engine();

            let cx = Context::now(&engine.shared, Scope::Cleanup).unwrap();

            for path in [
                "/Applications",
                "/Applications/A.app",
                "/Applications/Utilities/A.app",
                "/Applications/A.app/Contents",
                "/home/Applications",
                "/home/Applications/A.app",
            ] {
                assert_eq!(check(&cx, path), Err(Protected::Never), "{path}");
            }
            assert_eq!(check(&cx, "/home/Caches/x"), Ok(()));
            assert_eq!(check(&cx, "/Applicationsx/A.app"), Ok(()));
            assert!(cx.borrow().roots.contains(&PathBuf::from("/Applications")));
        }

        #[test]
        fn an_ordinary_cleanup_refuses_the_cleaners_app_folders_the_system_does_not_locate() {
            let opening = fresher_core::Protection::new(vec!["/Applications".into()], vec![], vec![])
                .and_then(|protection| protection.with_app_folders(&["/Applications".into()]))
                .unwrap();
            let system = fresher_core::SystemPorts {
                known_folders: std::sync::Arc::new(
                    fresher_testing::FakeKnownFolders::new().with(fresher_core::KnownFolder::Home, "/home"),
                ),
                ..crate::apps::fixture::system(
                    std::sync::Arc::new(fresher_testing::FakeInstalledApps::new(vec![])),
                    crate::apps::fixture::idle(),
                )
            };
            let cleaner = fresher_testing::FakeCleaner::default().protecting(opening);
            let engine = crate::testing::Fixture::new(crate::testing::ScriptedWalk::new())
                .system(system)
                .cleaner(std::sync::Arc::new(cleaner))
                .engine();

            let cleanup = Context::now(&engine.shared, Scope::Cleanup).unwrap();
            let uninstall = Context::now(&engine.shared, Scope::Uninstall).unwrap();

            for path in ["/Applications", "/Applications/A.app", "/Applications/Utilities/A.app"] {
                assert_eq!(check(&cleanup, path), Err(Protected::Never), "{path}");
                assert_eq!(check(&uninstall, path), Ok(()), "{path}");
            }
        }

        #[test]
        fn put_back_adds_no_refusal() {
            let engine = engine();

            let cx = Context::now(&engine.shared, Scope::PutBack).unwrap();

            assert_eq!(check(&cx, "/Applications/A.app"), Ok(()));
            assert_eq!(check(&cx, "/home/Applications/A.app"), Ok(()));
            assert_eq!(check(&cx, "/Applications"), Ok(()));
        }
    }

    mod positive {
        use std::path::PathBuf;

        use super::{Context, Scope, check, engine};

        #[test]
        fn an_uninstall_context_names_the_apps_folder_and_does_not_refuse_it() {
            let engine = engine();

            let cx = Context::now(&engine.shared, Scope::Uninstall).unwrap();

            assert_eq!(check(&cx, "/Applications/A.app"), Ok(()));
            assert_eq!(check(&cx, "/home/Applications/A.app"), Ok(()));
            let roots = cx.borrow().roots;
            assert!(roots.contains(&PathBuf::from("/Applications")));
            // The default root `~` holds the home's apps folder already.
            assert!(!roots.contains(&PathBuf::from("/home/Applications")));
        }

        #[test]
        fn a_root_is_read_with_the_home() {
            let home = Some(std::path::Path::new("/home"));
            let roots = [PathBuf::from("~"), PathBuf::from("/Volumes/x")];

            assert!(super::super::holds(&roots, "/home/Applications".as_ref(), home));
            assert!(super::super::holds(&roots, "/Volumes/x/A.app".as_ref(), home));
            assert!(!super::super::holds(&roots, "/Applications".as_ref(), home));
            assert!(!super::super::holds(&roots[..1], "/home/Applications".as_ref(), None));
        }
    }
}
