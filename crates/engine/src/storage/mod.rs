//! The storage map: where the bytes of a committed state went, by meaning and by place, built from the tree's own
//! sizes and the owners the scan recorded on it.

pub(crate) mod attribution;
mod meaning;
mod place;
mod query;
pub(crate) mod roots;
pub(crate) mod shares;
mod volume;
