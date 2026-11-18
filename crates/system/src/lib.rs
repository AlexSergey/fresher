//! What Fresher knows of the system, behind the core's ports: where the system keeps things (`MacKnownFolders`), the
//! installed apps by bundle id and team id (`MacInstalledApps`, with its own `Info.plist` and signature readers), the
//! running apps (`MacRunningApps`), the clock (`SystemClock`), external programs (`ProcessCommandRunner`).
//!
//! Safe Rust only: the few calls into the system that need FFI (the process table) live in `fresher-macos`.

#![forbid(unsafe_code)]

mod apps;
mod bundle;
mod clock;
mod known_folders;
mod plist;
mod process;
mod running;
mod signature;
mod spotlight;

pub use apps::{AppSearch, DeclaredSearch, MacInstalledApps};
pub use clock::{StartedClock, SystemClock};
pub use known_folders::MacKnownFolders;
pub use process::{COMMAND_TIMEOUT, ProcessCommandRunner};
pub use running::MacRunningApps;
pub use spotlight::{INDEX_CHECK_TIMEOUT, MacMounts, Mounts, SpotlightSearch};
