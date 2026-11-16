//! Reading folders with the bulk read: one system call per batch of entries.

mod parse;
mod reader;

pub use reader::MacFolderReader;
