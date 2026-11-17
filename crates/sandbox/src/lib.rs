//! Development sandboxes of Fresher: the case file of a feature — its negative and positive cases of files, and what
//! the app must do with each — read and checked (docs/plans/007-safe-development/00-design.md, "The case file"), the
//! layout of a sandbox folder, and the generator that writes the case files into one with its `EXPECTED.md` and its
//! world (`world.toml`: the moment it was generated and the apps that are open).
//!
//! Dev only: the app takes it as a normal dependency only because Cargo cannot gate a dependency on
//! `debug_assertions`; the code that uses it is compiled into development builds only (docs/hacks/README.md, "The
//! demo module linked in release builds").

#![forbid(unsafe_code)]

mod case;
mod error;
mod expected;
mod generate;
mod layout;
mod world;

pub use case::{AfterCleanup, Case, CaseFile, CaseKind, Expectation, LeftOutExpectation, MapExpectation, MapPartName};
pub use error::{CaseError, SandboxError};
pub use generate::{Generated, generate};
pub use layout::Sandbox;
pub use world::World;
