//! What Fresher knows of the system, behind the core's ports: where the system keeps things (`MacKnownFolders`), the
//! installed apps by bundle id and team id (`MacInstalledApps`, with its own `Info.plist` and signature readers), the
//! running apps (`MacRunningApps`), the clock (`SystemClock`), external programs (`ProcessCommandRunner`), what the
//! system's privacy protection lets the app see (`MacAccess`), System Settings and Finder (`MacSystemUi`), Finder's
//! settings (`MacFinderSettings`), the version of macOS (`MacosVersion`, `macos_version`), and the installer receipts
//! (`MacInstallRecords`).
//!
//! Safe Rust only: the few calls into the system that need FFI (the process table) live in `fresher-macos`.

#![forbid(unsafe_code)]

mod access;
mod apps;
mod bundle;
mod bundle_facts;
mod clock;
mod finder_settings;
mod guarded_root;
mod installs;
mod known_folders;
mod macos_version;
mod plist;
mod process;
mod running;
mod signature;
mod spotlight;
mod system_ui;

pub use access::MacAccess;
pub use apps::{AppSearch, DeclaredSearch, MacInstalledApps};
pub use bundle_facts::BundleReader;
pub use clock::{StartedClock, SystemClock};
pub use finder_settings::{MacFinderSettings, is_trash_setting_read};
pub use installs::MacInstallRecords;
pub use known_folders::MacKnownFolders;
pub use macos_version::{MacosVersion, macos_version};
pub use process::{COMMAND_TIMEOUT, ProcessCommandRunner};
pub use running::MacRunningApps;
pub use spotlight::{INDEX_CHECK_TIMEOUT, MacMounts, Mounts, SpotlightSearch};
pub use system_ui::MacSystemUi;
