//! The engine of Fresher: the module registry, the event bus, the scan session, the settings in memory, and the
//! cleanups: the plan the user confirms, the cleanup it starts, the history and put back.
//!
//! The engine depends on the core alone: the walk, the settings file, the clock, external commands, the access status,
//! the system's interface and facts, the cleaner, the cleanup journal and the volumes are ports, implemented elsewhere and
//! injected by the app.

#![forbid(unsafe_code)]

mod access;
mod batch;
#[doc(hidden)]
pub mod bench;
mod bus;
mod claims;
mod cleanup;
mod coverage;
mod disk;
mod engine;
mod error;
#[cfg(test)]
mod facts_tests;
mod ignore;
#[cfg(test)]
mod ignore_module_tests;
#[cfg(test)]
mod left_out_tests;
mod left_outs;
mod places;
mod projects;
mod registry;
mod results;
mod reveal;
mod session;
mod settings;
mod state;
mod storage;
#[cfg(test)]
mod testing;
mod throttle;

pub use bus::EventBus;
pub use engine::{CleanupPorts, Engine, Ports, ScanMode};
pub use registry::{Registry, RegistryError};
