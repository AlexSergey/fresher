//! The module's categories, their danger levels and their needs (design, sections 6 and 7): code, since a category id
//! is a `&'static str`; `toolchains.toml` names them by id and is checked against this list.

use fresher_core::{Capability, DangerLevel, GuardedFolder, Need};

/// Every category, in the order the module lists them: Rust toolchains (Low: one command brings one back), then
/// Node versions (Medium: their global packages go with them).
pub(crate) const CATEGORIES: [(&str, DangerLevel); 2] =
    [("unused-toolchains.rust", DangerLevel::Low), ("unused-toolchains.node", DangerLevel::Medium)];

/// What both categories need: the folders a project's pin file may be in, so that none is missed for want of access.
pub(crate) fn needs() -> Vec<Need> {
    [GuardedFolder::Desktop, GuardedFolder::Documents, GuardedFolder::Downloads]
        .into_iter()
        .map(|folder| Need::required(Capability::Folder(folder)))
        .collect()
}
