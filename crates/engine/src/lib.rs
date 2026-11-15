//! The engine of Mac Fresher: the module registry, the event bus, the scan session and the settings in memory.
//!
//! The engine depends on the core alone: the walk, the settings file, the clock, external commands and the access
//! status are ports, implemented elsewhere and injected by the app.

mod batch;
mod bus;
mod engine;
mod error;
mod registry;
mod results;
mod session;
mod settings;
mod state;
#[cfg(test)]
mod testing;
mod throttle;

pub use bus::EventBus;
pub use engine::{Engine, Ports};
pub use registry::{Registry, RegistryError};
