//! A demo scenario (`fixtures/demos/scenarios/<name>.toml`): its world, its window and the steps the UI's player
//! plays, from a closed list, checked here and sent to the UI as typed values
//! (docs/plans/038-demo-recordings/00-design.md, "The scenario and the player").

use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// The window's width a scenario may ask for, in logical points.
const WIDTHS: std::ops::RangeInclusive<u32> = 900..=1600;
/// The window's height a scenario may ask for, in logical points.
const HEIGHTS: std::ops::RangeInclusive<u32> = 600..=1000;
/// The length of one pause, in milliseconds.
const PAUSES: std::ops::RangeInclusive<u32> = 1..=10_000;
/// The most the pauses of a scenario may add up to, in milliseconds: the length a recording may have.
const MOST_PAUSED_MS: u64 = 20_000;

/// A demo scenario: the world it is played on, the window's size and the steps the player plays, in order.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct Scenario {
    pub name: String,
    pub world: String,
    pub window: DemoWindow,
    pub steps: Vec<Step>,
}

/// The size of the app's window during a demo, in logical points.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, TS)]
pub struct DemoWindow {
    pub width: u32,
    pub height: u32,
}

/// One step of a scenario.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(tag = "kind", rename_all = "kebab-case", rename_all_fields = "camelCase")]
pub enum Step {
    /// The pointer glides to the target and presses it.
    Click { target: Target },
    /// The pointer glides to the target and rests on it.
    Hover { target: Target },
    /// The target is scrolled `by` pixels, down when positive, never 0.
    Scroll { target: Target, by: i32 },
    /// The player waits until the app is in a state or a target appears.
    Wait { until: Wait },
    /// The player waits `ms` milliseconds, 1 to 10 000.
    Pause { ms: u32 },
}

/// An element of the interface, as the e2e tests address one: its accessible role and name.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct Target {
    pub role: Role,
    pub name: NameMatch,
}

/// The accessible role of a target.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "kebab-case")]
pub enum Role {
    Button,
    Link,
    Tab,
    Checkbox,
    Switch,
    Heading,
    Region,
    Dialog,
    Row,
}

/// How a target's accessible name is matched; the text is never empty.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum NameMatch {
    /// The whole accessible name.
    Exact { text: String },
    /// The start of the accessible name, for a name that carries a size.
    Start { text: String },
}

/// What a wait step waits for.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum Wait {
    /// A named state of the app.
    State { state: AppState },
    /// A target that appears.
    Target { target: Target },
}

/// A named state of the app a scenario can wait for.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "kebab-case")]
pub enum AppState {
    /// A scan has finished.
    ScanFinished,
    /// A cleanup has finished.
    CleanupFinished,
}

/// What the shell answers the UI in demo mode: the scenario, and the texts the picture must never show (the real
/// home folder's path and the sandbox folder's path).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct DemoPlay {
    pub scenario: Scenario,
    pub private: Vec<String>,
}

/// How a played scenario ended, as the UI reports it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "kebab-case", rename_all_fields = "camelCase")]
pub enum DemoOutcome {
    /// Every step was played, in `elapsed_ms` milliseconds.
    Finished { elapsed_ms: u32 },
    /// The step `step`, counted from 1, failed for `reason`.
    Failed { step: u32, reason: String },
}

/// Why a scenario was refused; an error about one step names it, counted from 1.
#[derive(Debug, thiserror::Error)]
pub enum ScenarioError {
    /// The file could not be read.
    #[error("{path}: {source}")]
    Io { path: PathBuf, source: io::Error },
    /// The file is not TOML of the scenario format (an unknown key included).
    #[error("the scenario is not valid: {source}")]
    Format { source: Box<toml::de::Error> },
    /// The world is not a plain name.
    #[error("the world `{world}` is not a plain name of letters, digits and `-`")]
    BadWorld { world: String },
    /// The window is outside 900–1600 × 600–1000 logical points.
    #[error("the window {width} × {height} is outside 900–1600 × 600–1000 points")]
    Window { width: u32, height: u32 },
    /// The scenario has no step.
    #[error("the scenario has no step")]
    NoSteps,
    /// A step has none or more than one of `click`, `hover`, `scroll`, `wait`, `pause`.
    #[error("step {step}: a step has exactly one of `click`, `hover`, `scroll`, `wait` and `pause`")]
    StepKeys { step: u32 },
    /// A target has both `name` and `name-start`, neither, or an empty text.
    #[error("step {step}: a target has either `name` or `name-start`, with a text")]
    Name { step: u32 },
    /// A scroll by 0 pixels.
    #[error("step {step}: a scroll's `by` is never 0")]
    ScrollZero { step: u32 },
    /// A pause outside 1–10 000 milliseconds.
    #[error("step {step}: a pause of {ms} ms is outside 1–10000 ms")]
    Pause { step: u32, ms: u32 },
    /// The pauses add up to more than 20 000 milliseconds.
    #[error("the pauses add up to {total_ms} ms, more than {MOST_PAUSED_MS} ms")]
    TooLong { total_ms: u64 },
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawScenario {
    world: String,
    window: RawWindow,
    #[serde(default)]
    step: Vec<RawStep>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawWindow {
    width: u32,
    height: u32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawStep {
    click: Option<RawTarget>,
    hover: Option<RawTarget>,
    scroll: Option<RawScroll>,
    wait: Option<RawWait>,
    pause: Option<u32>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
struct RawTarget {
    role: Role,
    name: Option<String>,
    name_start: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
struct RawScroll {
    role: Role,
    name: Option<String>,
    name_start: Option<String>,
    by: i32,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum RawWait {
    State(AppState),
    Target(RawTarget),
}

impl Scenario {
    /// The scenario of the file at `path`, named by its file stem.
    pub fn read(path: &Path) -> Result<Self, ScenarioError> {
        let text = std::fs::read_to_string(path).map_err(|source| ScenarioError::Io { path: path.into(), source })?;
        let name = path.file_stem().map_or_else(String::new, |stem| stem.to_string_lossy().into_owned());
        Self::parse(&name, &text)
    }

    /// The scenario `name` of the TOML `text`, checked: a plain world name, a window within bounds, at least one step,
    /// each with exactly one key and valid targets, and pauses within bounds.
    pub fn parse(name: &str, text: &str) -> Result<Self, ScenarioError> {
        let raw: RawScenario =
            toml::from_str(text).map_err(|source| ScenarioError::Format { source: Box::new(source) })?;
        if !is_plain_name(&raw.world) {
            return Err(ScenarioError::BadWorld { world: raw.world });
        }
        let RawWindow { width, height } = raw.window;
        if !WIDTHS.contains(&width) || !HEIGHTS.contains(&height) {
            return Err(ScenarioError::Window { width, height });
        }
        if raw.step.is_empty() {
            return Err(ScenarioError::NoSteps);
        }
        let steps = (1..).zip(raw.step).map(|(number, step)| step.checked(number)).collect::<Result<Vec<_>, _>>()?;
        let total_ms: u64 =
            steps.iter().map(|step| if let Step::Pause { ms } = step { u64::from(*ms) } else { 0 }).sum();
        if total_ms > MOST_PAUSED_MS {
            return Err(ScenarioError::TooLong { total_ms });
        }
        Ok(Self { name: name.into(), world: raw.world, window: DemoWindow { width, height }, steps })
    }
}

impl RawStep {
    /// The typed step numbered `step`.
    fn checked(self, step: u32) -> Result<Step, ScenarioError> {
        let checked = match self {
            Self { click: Some(raw), hover: None, scroll: None, wait: None, pause: None } => {
                Step::Click { target: raw.checked(step)? }
            }
            Self { click: None, hover: Some(raw), scroll: None, wait: None, pause: None } => {
                Step::Hover { target: raw.checked(step)? }
            }
            Self { click: None, hover: None, scroll: Some(raw), wait: None, pause: None } => {
                if raw.by == 0 {
                    return Err(ScenarioError::ScrollZero { step });
                }
                Step::Scroll { target: target(step, raw.role, raw.name, raw.name_start)?, by: raw.by }
            }
            Self { click: None, hover: None, scroll: None, wait: Some(raw), pause: None } => Step::Wait {
                until: match raw {
                    RawWait::State(state) => Wait::State { state },
                    RawWait::Target(raw) => Wait::Target { target: raw.checked(step)? },
                },
            },
            Self { click: None, hover: None, scroll: None, wait: None, pause: Some(ms) } => {
                if !PAUSES.contains(&ms) {
                    return Err(ScenarioError::Pause { step, ms });
                }
                Step::Pause { ms }
            }
            _ => return Err(ScenarioError::StepKeys { step }),
        };
        Ok(checked)
    }
}

impl RawTarget {
    fn checked(self, step: u32) -> Result<Target, ScenarioError> {
        target(step, self.role, self.name, self.name_start)
    }
}

/// The target of `role` with exactly one of `name` and `name_start`, whose text is not blank.
fn target(step: u32, role: Role, name: Option<String>, name_start: Option<String>) -> Result<Target, ScenarioError> {
    let name = match (name, name_start) {
        (Some(text), None) if !text.trim().is_empty() => NameMatch::Exact { text },
        (None, Some(text)) if !text.trim().is_empty() => NameMatch::Start { text },
        _ => return Err(ScenarioError::Name { step }),
    };
    Ok(Target { role, name })
}

/// Whether `name` is a plain name, as a world or a scenario is named: not empty, of ASCII letters, digits and `-` only.
pub fn is_plain_name(name: &str) -> bool {
    !name.is_empty() && name.chars().all(|character| character.is_ascii_alphanumeric() || character == '-')
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod traced;
