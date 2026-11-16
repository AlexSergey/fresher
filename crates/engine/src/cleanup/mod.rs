//! Cleanups: the plan builder and the pending plan, the clean operation, the history and put back.

mod blocked;
mod context;
mod history;
mod operation;
mod pending;
mod plan;
mod view;

#[cfg(test)]
pub(crate) use context::Context;
pub(crate) use history::{detail, page, put_back, reveal};
pub(crate) use operation::confirm;
pub(crate) use pending::{Pending, prepare};
#[cfg(test)]
pub(crate) mod testing;
