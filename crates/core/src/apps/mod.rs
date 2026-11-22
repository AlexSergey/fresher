//! The installed apps as the Apps screen shows them: a bundle's facts, its footprint's kinds, the uninstall choices
//! and their categories, and the IPC types of the list and an app's page.

mod bundle;
mod categories;
mod kinds;
mod view;

pub use bundle::{AppsError, BundleFacts, InstalledBy, Manager};
pub use categories::{uninstall_categories, uninstall_category, uninstall_title};
pub use kinds::{FootprintKind, KeptReason, NotOffered, UninstallChoice};
pub use view::{
    AppDetail, AppId, AppOffer, AppSummary, AppsQuery, AppsView, ChoiceView, FootprintItem, KeptItem, LastUse,
    PartSizes, UnusedFor,
};
