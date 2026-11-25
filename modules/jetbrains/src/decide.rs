//! Whether a version folder of an IDE is offered or kept: the rows of the design's section 4, in their order, on plain
//! values. Every row but the fourth and the last keeps the folder; the fourth waits for every folder of the scan
//! (`uninstalled.rs`); nothing here reads the disk.

use fresher_core::{BundleId, InstalledApp, LeftOutKind, RunningApp};

use crate::name::{Version, installed_version};
use crate::products::{Product, Products};

/// What becomes of one version folder.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Verdict {
    /// Offered: an installed, closed IDE of the product uses `current`, newer than the folder's; `app` is its bundle
    /// id, the newest installed app of the product.
    Offer { current: Version, app: BundleId },
    /// Kept, and reported as left out with the reason.
    LeftOut(Reason),
    /// No app of the product is installed, the installed list is complete and nothing of the vendor's runs: offered
    /// as the data of an IDE that is no longer installed when every folder of the product is old enough, which is
    /// known only once the walk has seen them all (`uninstalled.rs`).
    Uninstalled,
}

/// Why a folder the module recognized is kept (design, section 5).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Reason {
    Current,
    Newer,
    /// An app with one of the product's bundle ids runs.
    Running,
    /// An app of the vendor that the table does not know runs: it may be an IDE that uses the folders.
    VendorAppOpen,
    UnknownVersion,
    /// The installed or the running apps cannot be listed.
    NoSystem,
    /// The installed apps are listed, but an apps folder, a folder in one or a bundle could not be read: an IDE may be
    /// among them.
    AppsUnread,
    /// The product is not in the apps folders, and Spotlight cannot tell whether an app of it is elsewhere.
    SpotlightUnread,
    /// The product is not installed, but one of its folders changed within the setting's days, the scan is restored
    /// from the cache, which tells no age, or the walk could not read every folder, so one unseen may have changed.
    UsedRecently,
    /// The product is not installed, but an app of the vendor the table does not know is: it may use the folders.
    UnknownIde,
    /// The product is not in the apps folders, but Spotlight knows an app of it elsewhere.
    InstalledElsewhere,
}

impl Reason {
    /// Every reason, in the order the module declares their keys.
    pub(crate) const ALL: [Self; 11] = [
        Self::Current,
        Self::Newer,
        Self::Running,
        Self::UnknownVersion,
        Self::NoSystem,
        Self::UsedRecently,
        Self::UnknownIde,
        Self::InstalledElsewhere,
        Self::VendorAppOpen,
        Self::AppsUnread,
        Self::SpotlightUnread,
    ];

    /// Whether the folder is kept for good (the version you use, a newer one's, or an installed IDE's) or only for
    /// now (an IDE is open, a list could not be read, it changed lately).
    pub(crate) const fn kind(self) -> LeftOutKind {
        match self {
            Self::Current | Self::Newer | Self::UnknownIde | Self::InstalledElsewhere => LeftOutKind::Kept,
            Self::Running
            | Self::VendorAppOpen
            | Self::UnknownVersion
            | Self::NoSystem
            | Self::AppsUnread
            | Self::SpotlightUnread
            | Self::UsedRecently => LeftOutKind::ForNow,
        }
    }

    /// Whether the reason is about an IDE that is not installed: its left-out goes under that category.
    pub(crate) const fn of_removed_ide(self) -> bool {
        matches!(self, Self::UsedRecently | Self::UnknownIde | Self::InstalledElsewhere | Self::SpotlightUnread)
    }
}

/// The first row that applies to the folder of `product` at `folder`:
///
/// 1. the installed or the running apps cannot be listed: `NoSystem`; the installed list is partial: `AppsUnread`
///    (the caller tells which, as the error of `installed` and `running`);
/// 2. an app with one of the product's bundle ids runs: `Running`;
/// 3. an app of the vendor that the table does not know runs (it may be an IDE): `VendorAppOpen`;
/// 4. no installed app has one of the product's bundle ids: `UnknownIde` when an app of the vendor the table does not
///    know is installed, else `Uninstalled`, decided by age (and Spotlight) once every folder is seen;
/// 5. an installed app of the product has a version that cannot be read: `UnknownVersion`;
/// 6. the folder's version is one an installed app of the product has: `Current`;
/// 7. it is newer than the newest installed one: `Newer`;
/// 8. it is older: offered.
pub(crate) fn decide(
    products: &Products,
    product: &Product,
    folder: Version,
    installed: Result<&[InstalledApp], Reason>,
    running: Result<&[RunningApp], Reason>,
) -> Verdict {
    let running = match running {
        Ok(running) => running,
        Err(reason) => return Verdict::LeftOut(reason),
    };
    let installed = match installed {
        Ok(installed) => installed,
        Err(reason) => return Verdict::LeftOut(reason),
    };
    let ours = |id: &BundleId| product.bundle_ids().contains(id);
    let running_ids = || running.iter().flat_map(|app| &app.bundle_ids);
    if running_ids().any(ours) {
        return Verdict::LeftOut(Reason::Running);
    }
    if running_ids().any(|id| products.is_unknown_vendor_app(id)) {
        return Verdict::LeftOut(Reason::VendorAppOpen);
    }
    let apps: Vec<&InstalledApp> = installed.iter().filter(|app| ours(&app.bundle_id)).collect();
    if apps.is_empty() {
        if installed.iter().any(|app| products.is_unknown_vendor_app(&app.bundle_id)) {
            return Verdict::LeftOut(Reason::UnknownIde);
        }
        return Verdict::Uninstalled;
    }
    let versions: Option<Vec<(Version, &BundleId)>> = apps
        .iter()
        .map(|app| app.version.as_deref().and_then(installed_version).map(|version| (version, &app.bundle_id)))
        .collect();
    let Some(versions) = versions else {
        return Verdict::LeftOut(Reason::UnknownVersion);
    };
    if versions.iter().any(|&(version, _)| version == folder) {
        return Verdict::LeftOut(Reason::Current);
    }
    // The first of the newest, so two apps of one version name the same app whatever else is installed.
    let newest = versions.into_iter().reduce(|best, next| if next.0 > best.0 { next } else { best });
    match newest {
        Some((current, app)) if folder < current => Verdict::Offer { current, app: app.clone() },
        // `None` is unreachable (`apps` is not empty, so neither is `versions`); it keeps the folder all the same.
        Some(_) | None => Verdict::LeftOut(Reason::Newer),
    }
}

#[cfg(test)]
mod tests {
    use fresher_core::{InstalledApp, RunningApp};
    use fresher_testing::{installed_app, running_app};

    use crate::decide::{Reason, Verdict, decide};
    use crate::name::Version;
    use crate::products::Products;

    fn version(year: u16, minor: u8) -> Version {
        Version::new(year, minor).unwrap()
    }

    fn app(bundle_id: &str, version: &str) -> InstalledApp {
        installed_app(bundle_id, bundle_id, version).unwrap()
    }

    fn open(bundle_id: &str) -> RunningApp {
        running_app(bundle_id, &[bundle_id]).unwrap()
    }

    /// The verdict on the `WebStorm` folder of `folder` with these apps installed and running.
    fn webstorm(folder: Version, installed: &[InstalledApp], running: &[RunningApp]) -> Verdict {
        on("WebStorm", folder, Ok(installed), Ok(running))
    }

    fn on(
        product: &str,
        folder: Version,
        installed: Result<&[InstalledApp], Reason>,
        running: Result<&[RunningApp], Reason>,
    ) -> Verdict {
        let products = Products::shipped().unwrap();
        let product = products.by_folder(product).unwrap();
        decide(&products, product, folder, installed, running)
    }

    mod negative {
        use fresher_core::InstalledApp;

        use super::{app, on, open, version, webstorm};
        use crate::decide::{Reason, Verdict};

        #[test]
        fn row_1_keeps_every_folder_when_either_list_fails() {
            let installed = [app("com.jetbrains.WebStorm", "2025.3")];
            let failed = || Reason::NoSystem;

            for (apps, running) in
                [(Err(failed()), Ok(&[][..])), (Ok(&installed[..]), Err(failed())), (Err(failed()), Err(failed()))]
            {
                assert_eq!(on("WebStorm", version(2024, 1), apps, running), Verdict::LeftOut(Reason::NoSystem));
            }
        }

        #[test]
        fn row_1_a_partial_list_keeps_every_folder_as_apps_unread_but_a_failed_running_list_wins() {
            let running = [open("com.jetbrains.WebStorm")];

            assert_eq!(
                on("WebStorm", version(2024, 1), Err(Reason::AppsUnread), Ok(&running)),
                Verdict::LeftOut(Reason::AppsUnread)
            );
            assert_eq!(
                on("WebStorm", version(2024, 1), Err(Reason::AppsUnread), Err(Reason::NoSystem)),
                Verdict::LeftOut(Reason::NoSystem)
            );
        }

        #[test]
        fn row_2_keeps_every_folder_of_a_running_product_installed_or_not() {
            let installed = [app("com.jetbrains.WebStorm", "2025.3")];

            for apps in [&installed[..], &[]] {
                assert_eq!(
                    webstorm(version(2024, 1), apps, &[open("com.jetbrains.WebStorm")]),
                    Verdict::LeftOut(Reason::Running)
                );
            }
        }

        #[test]
        fn row_2_an_early_access_bundle_id_counts_as_the_product() {
            let installed = [app("com.jetbrains.WebStorm", "2025.3")];

            assert_eq!(
                webstorm(version(2024, 1), &installed, &[open("com.jetbrains.webstorm-eap")]),
                Verdict::LeftOut(Reason::Running)
            );
        }

        #[test]
        fn row_3_an_unknown_vendor_app_running_keeps_every_products_folders() {
            let installed = [app("com.jetbrains.WebStorm", "2025.3")];

            assert_eq!(
                webstorm(version(2024, 1), &installed, &[open("com.jetbrains.NewIde")]),
                Verdict::LeftOut(Reason::VendorAppOpen)
            );
        }

        #[test]
        fn row_4_never_offers_at_once_a_product_with_no_installed_app() {
            let others = [app("com.jetbrains.pycharm", "2025.3"), app("com.example.WebStorm", "2025.3")];

            for folder in [version(2020, 1), version(2025, 3), version(2026, 1)] {
                assert_eq!(webstorm(folder, &others, &[]), Verdict::Uninstalled);
            }
        }

        #[test]
        fn row_4_the_toolbox_app_installed_is_not_an_unknown_ide() {
            let installed = [app("com.jetbrains.toolbox", "2.5")];

            assert_eq!(on("DataGrip", version(2024, 1), Ok(&installed), Ok(&[])), Verdict::Uninstalled);
        }

        #[test]
        fn row_4_the_community_edition_is_not_the_full_edition() {
            let installed = [app("com.jetbrains.pycharm", "2025.2")];

            assert_eq!(on("PyCharmCE", version(2024, 1), Ok(&installed), Ok(&[])), Verdict::Uninstalled);
        }

        #[test]
        fn row_4_an_ide_the_toolbox_installed_is_installed() {
            let mut toolbox = app("com.jetbrains.datagrip", "2025.3");
            toolbox.path =
                "/Users/me/Library/Application Support/JetBrains/Toolbox/apps/DataGrip/ch-0/253.1/DataGrip.app".into();

            assert_eq!(on("DataGrip", version(2025, 3), Ok(&[toolbox]), Ok(&[])), Verdict::LeftOut(Reason::Current));
        }

        #[test]
        fn row_4_an_unknown_vendor_app_installed_keeps_a_product_that_is_not() {
            let installed = [app("com.jetbrains.NewIde", "2026.1"), app("com.jetbrains.WebStorm", "2025.3")];

            assert_eq!(on("DataGrip", version(2024, 1), Ok(&installed), Ok(&[])), Verdict::LeftOut(Reason::UnknownIde));
        }

        #[test]
        fn rows_2_and_3_keep_a_product_that_is_not_installed() {
            for (running, reason) in [
                (open("com.jetbrains.datagrip"), Reason::Running),
                (open("com.jetbrains.datagrip-EAP"), Reason::Running),
                (open("com.jetbrains.Gateway"), Reason::VendorAppOpen),
            ] {
                assert_eq!(
                    on("DataGrip", version(2024, 1), Ok(&[]), Ok(std::slice::from_ref(&running))),
                    Verdict::LeftOut(reason),
                    "{running:?}"
                );
            }
        }

        #[test]
        fn row_5_keeps_every_folder_when_an_installed_version_cannot_be_read() {
            let unversioned = InstalledApp { version: None, ..app("com.jetbrains.WebStorm-EAP", "") };
            for installed in [
                vec![app("com.jetbrains.WebStorm", "2025.3"), unversioned],
                vec![app("com.jetbrains.WebStorm", "253.1234")],
                vec![app("com.jetbrains.WebStorm", "")],
            ] {
                assert_eq!(
                    webstorm(version(2024, 1), &installed, &[]),
                    Verdict::LeftOut(Reason::UnknownVersion),
                    "{installed:?}"
                );
            }
        }

        #[test]
        fn row_6_keeps_the_version_in_use() {
            let installed = [app("com.jetbrains.WebStorm", "2025.3.1")];

            assert_eq!(webstorm(version(2025, 3), &installed, &[]), Verdict::LeftOut(Reason::Current));
        }

        #[test]
        fn row_6_keeps_both_versions_installed_side_by_side() {
            let installed = [app("com.jetbrains.intellij", "2025.2"), app("com.jetbrains.intellij-EAP", "2025.3 EAP")];

            for folder in [version(2025, 2), version(2025, 3)] {
                assert_eq!(on("IntelliJIdea", folder, Ok(&installed), Ok(&[])), Verdict::LeftOut(Reason::Current));
            }
        }

        #[test]
        fn row_7_keeps_a_newer_folder() {
            let installed = [app("com.jetbrains.WebStorm", "2025.3")];

            for folder in [version(2025, 4), version(2025, 10), version(2026, 1)] {
                assert_eq!(webstorm(folder, &installed, &[]), Verdict::LeftOut(Reason::Newer));
            }
        }

        #[test]
        fn running_wins_over_not_installed_and_current() {
            let installed = [app("com.jetbrains.WebStorm", "2025.3")];
            let running = [open("com.jetbrains.WebStorm")];

            assert_eq!(webstorm(version(2025, 3), &installed, &running), Verdict::LeftOut(Reason::Running));
            assert_eq!(webstorm(version(2025, 3), &[], &running), Verdict::LeftOut(Reason::Running));
        }

        #[test]
        fn two_installed_apps_of_one_id_keep_both_versions_and_what_is_newer() {
            let installed = [app("com.jetbrains.WebStorm", "2025.1"), app("com.jetbrains.WebStorm", "2025.3")];

            for folder in [version(2025, 1), version(2025, 3)] {
                assert_eq!(webstorm(folder, &installed, &[]), Verdict::LeftOut(Reason::Current));
            }
            assert_eq!(webstorm(version(2026, 1), &installed, &[]), Verdict::LeftOut(Reason::Newer));
        }

        #[test]
        fn no_system_wins_over_running() {
            let running = [open("com.jetbrains.WebStorm")];

            assert_eq!(
                on("WebStorm", version(2024, 1), Err(Reason::NoSystem), Ok(&running)),
                Verdict::LeftOut(Reason::NoSystem)
            );
        }
    }

    mod positive {
        use fresher_core::BundleId;

        use super::{app, on, open, version, webstorm};
        use crate::decide::Verdict;

        fn offer(year: u16, minor: u8, app: &str) -> Verdict {
            Verdict::Offer { current: version(year, minor), app: BundleId::new(app).unwrap() }
        }

        #[test]
        fn row_8_offers_an_older_folder() {
            let installed = [app("com.jetbrains.WebStorm", "2025.3.1")];

            for folder in [version(2025, 2), version(2024, 3), version(2020, 1)] {
                assert_eq!(webstorm(folder, &installed, &[]), offer(2025, 3, "com.jetbrains.WebStorm"));
            }
        }

        #[test]
        fn offers_what_is_older_than_the_newest_of_two_side_by_side_and_names_the_newest() {
            let installed = [app("com.jetbrains.intellij", "2025.2"), app("com.jetbrains.intellij-EAP", "2025.3 EAP")];

            assert_eq!(
                on("IntelliJIdea", version(2024, 3), Ok(&installed), Ok(&[])),
                offer(2025, 3, "com.jetbrains.intellij-EAP")
            );
        }

        #[test]
        fn an_early_access_app_alone_counts_as_installed() {
            let installed = [app("com.jetbrains.WebStorm-EAP", "2026.1 EAP")];

            assert_eq!(webstorm(version(2025, 3), &installed, &[]), offer(2026, 1, "com.jetbrains.WebStorm-EAP"));
        }

        #[test]
        fn another_products_running_app_and_the_toolbox_keep_nothing() {
            let installed = [app("com.jetbrains.WebStorm", "2025.3")];
            let running = [open("com.jetbrains.pycharm"), open("com.jetbrains.toolbox"), open("com.example.Editor")];

            assert_eq!(webstorm(version(2024, 1), &installed, &running), offer(2025, 3, "com.jetbrains.WebStorm"));
        }

        #[test]
        fn two_installed_apps_of_one_id_offer_what_is_older_than_the_newest() {
            let installed = [app("com.jetbrains.WebStorm", "2025.3"), app("com.jetbrains.WebStorm", "2025.1")];

            assert_eq!(webstorm(version(2024, 3), &installed, &[]), offer(2025, 3, "com.jetbrains.WebStorm"));
        }

        #[test]
        fn two_apps_of_one_version_name_the_first() {
            let installed = [app("com.jetbrains.WebStorm", "2025.3"), app("com.jetbrains.WebStorm-EAP", "2025.3.2")];

            assert_eq!(webstorm(version(2024, 1), &installed, &[]), offer(2025, 3, "com.jetbrains.WebStorm"));
        }
    }
}
