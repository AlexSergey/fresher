//! The fast parallel file system scan of Fresher; read-only.

#![forbid(unsafe_code)]

#[doc(hidden)]
pub mod bench;

mod codec;
#[expect(dead_code, reason = "the walker of plan 004/05 uses the matcher")]
mod matcher;
mod places_seen;
mod rescan;
mod scanner;
mod state;
mod subtrees;
#[expect(dead_code, reason = "the walker of plan 004/05 builds the tree")]
mod tree;
mod walk;

pub use scanner::{AnchorResolver, Scanner, ScannerOptions, home_anchor, known_folders_anchor};
pub use walk::counting::CountingReader;
pub use walk::std_reader::StdReader;
