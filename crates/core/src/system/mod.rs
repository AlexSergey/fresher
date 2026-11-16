//! What the app knows of the system, shared by every module: the installed and running apps, and the projects the
//! walk found. The ports are implemented in `fresher-system`; the fakes are in `fresher-testing`.

mod apps;
mod facts;
mod ids;
mod projects;
mod running;

pub use apps::{InstalledApp, InstalledApps};
pub use facts::{SystemFacts, SystemPorts};
pub use ids::{AppIdError, BundleId, TeamId};
pub use projects::{Project, ProjectKind};
pub use running::{RunningApp, RunningApps, RunningSet};

#[cfg(test)]
pub(crate) use facts::nothing_known;

/// Why a fact of the system could not be gathered. Holds no path.
#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
#[error("the system could not be asked: {reason}")]
pub struct SystemError {
    pub reason: String,
}
