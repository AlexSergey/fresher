//! Development sandboxes of Fresher: the case file of a feature — its negative and positive cases of files, and what
//! the app must do with each — read and checked (docs/plans/007-safe-development/00-design.md, "The case file"), the
//! layout of a sandbox folder, and the generator that writes the case files into one with its `EXPECTED.md` and its
//! world (`world.toml`: the moment it was generated, the apps that are open, what Spotlight answers and the facts its
//! entries state). A demo world is a case file with a `[demo]` header and scenery cases: a believable Mac generated
//! at its own clock, whose content is shown, never checked, on the volume it states ([`StatedSpace`])
//! (docs/plans/038-demo-recordings/00-design.md).
//!
//! Dev only: the app takes it as a normal dependency only because Cargo cannot gate a dependency on
//! `debug_assertions`; the code that uses it is compiled into development builds only (docs/hacks/README.md, "The
//! demo module linked in release builds").

#![forbid(unsafe_code)]

mod case;
mod demo;
mod error;
mod expected;
mod generate;
mod layout;
mod scenario;
mod stated_space;
mod world;

pub use case::{
    AfterCleanup, Case, CaseFile, CaseKind, Expectation, InsightExpectation, LeftOutExpectation, MapExpectation,
    MapPartName, merged_settings,
};
pub use demo::{Demo, DemoSizes, DemoVolume};
pub use error::{CaseError, SandboxError};
pub use generate::{Generated, LeftOutFile, generate, generate_leaving_out, without_clashes};
pub use layout::Sandbox;
pub use scenario::{
    AppState, DemoOutcome, DemoPlay, DemoWindow, NameMatch, Role, Scenario, ScenarioError, Step, Target, Wait,
    is_plain_name,
};
pub use stated_space::StatedSpace;
pub use world::{World, WorldCommands};
