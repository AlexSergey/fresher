//! Cleanups: the plan builder and the pending plan, the clean operation, the history and put back.

mod blocked;
mod context;
mod history;
mod operation;
mod pending;
mod plan;
mod trash;
#[cfg(test)]
mod trash_tests;
mod view;

#[cfg(test)]
pub(crate) use context::{Context, Scope};
pub(crate) use context::{apps_folders, holds};
pub(crate) use history::{detail, page, put_back, reveal};
pub(crate) use operation::confirm;
pub(crate) use pending::{Pending, Planned, Purpose, prepare, rehearse_and_keep};
pub(crate) use trash::{open as open_trash, view as trash_view};
#[cfg(test)]
pub(crate) mod testing;
