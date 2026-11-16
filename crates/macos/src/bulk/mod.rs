//! Reading folders with the bulk read: one system call per batch of entries.

mod parse;
mod reader;

pub(crate) use parse::device_id;
pub use reader::MacFolderReader;
pub(crate) use reader::read_held;
