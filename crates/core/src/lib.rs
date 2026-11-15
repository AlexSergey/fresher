//! Domain types of Mac Fresher: categories, danger levels, findings, sizes and clean plans.
//!
//! The ports to the outside world (the clock, the Trash, external commands) are traits here, so every other crate can
//! be tested with the fakes of `fresher-testing`.

mod ports;
mod size;

pub use ports::{Clock, CommandError, CommandOutput, CommandRunner, Trash, TrashError};
pub use size::ByteSize;
