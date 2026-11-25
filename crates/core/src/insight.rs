//! What a module describes and never offers: figures, parts and the commands the user may run by hand.

use serde::Serialize;
use ts_rs::TS;

use crate::ids::is_kebab_case;
use crate::{ByteSize, DangerLevel, Text, ToolCommand, ToolId};

/// The most figures one insight may hold.
pub const MAX_FIGURES: usize = 4;
/// The most parts one insight may hold.
pub const MAX_INSIGHT_PARTS: usize = 8;
/// The most notes one insight may hold.
pub const MAX_NOTES: usize = 4;

/// Space a module describes and never offers (`FindingSink::describe`): never selectable, never counted in a total,
/// never stored or cached. It travels only in the module's `ModuleFinished`; a restored scan describes again.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct Insight {
    identity: String,
    title: Text,
    summary: Text,
    figures: Vec<Figure>,
    parts: Vec<InsightPart>,
    notes: Vec<Text>,
}

/// One number of an insight with its label (the disk file's maximum, the space it takes).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct Figure {
    pub label: Text,
    pub size: ByteSize,
}

/// One part of what an insight describes: its size, what it is, what removing it would do, how dangerous that is and
/// the command the user may run by hand.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct InsightPart {
    pub label: Text,
    pub size: ByteSize,
    pub detail: Text,
    pub effect: Text,
    pub danger: DangerLevel,
    pub command: Option<ShownCommand>,
}

/// A command the app shows and the user may copy; the app never runs it. It never forces and never needs quoting, so
/// its `line` pastes into a shell as it is.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ShownCommand {
    tool: ToolId,
    args: Vec<String>,
    line: String,
}

/// Why an insight or a shown command was refused.
#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub enum InsightError {
    #[error("a shown command has no argument")]
    NoArguments,
    #[error("the argument {arg:?} of a shown command is empty or needs quoting")]
    NeedsQuoting { arg: String },
    #[error("the argument {arg:?} of a shown command forces the tool")]
    Forced { arg: String },
    #[error("the insight identity {identity:?} is not kebab case")]
    Identity { identity: String },
    #[error("an insight holds {count} figures, more than {MAX_FIGURES}")]
    TooManyFigures { count: usize },
    #[error("an insight holds {count} parts, more than {MAX_INSIGHT_PARTS}")]
    TooManyParts { count: usize },
    #[error("an insight holds {count} notes, more than {MAX_NOTES}")]
    TooManyNotes { count: usize },
}

impl ShownCommand {
    /// `tool` with `args`, refused when there is no argument, an argument is empty or holds a character outside ASCII
    /// letters, digits and `. _ / : = @ + -`, or an argument forces the tool (the rule of `ToolCommand::forces`).
    pub fn new(tool: ToolId, args: Vec<String>) -> Result<Self, InsightError> {
        if args.is_empty() {
            return Err(InsightError::NoArguments);
        }
        if let Some(arg) = args.iter().find(|arg| !is_plain_argument(arg)) {
            return Err(InsightError::NeedsQuoting { arg: arg.clone() });
        }
        let forces = |arg: &String| ToolCommand::new(tool, vec![arg.clone()]).is_ok_and(|command| command.forces());
        if let Some(arg) = args.iter().find(|arg| forces(arg)) {
            return Err(InsightError::Forced { arg: arg.clone() });
        }
        let line = std::iter::once(tool.as_str()).chain(args.iter().map(String::as_str)).collect::<Vec<_>>().join(" ");
        Ok(Self { tool, args, line })
    }

    pub const fn tool(&self) -> ToolId {
        self.tool
    }

    pub fn args(&self) -> &[String] {
        &self.args
    }

    /// The tool's id and the arguments joined by single spaces, as the UI shows and copies it.
    pub fn line(&self) -> &str {
        &self.line
    }
}

/// `true` for a non-empty argument a shell takes as it is.
fn is_plain_argument(arg: &str) -> bool {
    !arg.is_empty() && arg.chars().all(|char| char.is_ascii_alphanumeric() || "._/:=@+-".contains(char))
}

impl Insight {
    /// An insight called `identity` (kebab case, unique within its module), refused when the identity is not kebab
    /// case or the figures, parts or notes are more than their maximum.
    pub fn new(
        identity: &str,
        title: Text,
        summary: Text,
        figures: Vec<Figure>,
        parts: Vec<InsightPart>,
        notes: Vec<Text>,
    ) -> Result<Self, InsightError> {
        if !is_kebab_case(identity) {
            return Err(InsightError::Identity { identity: identity.into() });
        }
        if figures.len() > MAX_FIGURES {
            return Err(InsightError::TooManyFigures { count: figures.len() });
        }
        if parts.len() > MAX_INSIGHT_PARTS {
            return Err(InsightError::TooManyParts { count: parts.len() });
        }
        if notes.len() > MAX_NOTES {
            return Err(InsightError::TooManyNotes { count: notes.len() });
        }
        Ok(Self { identity: identity.into(), title, summary, figures, parts, notes })
    }

    pub fn identity(&self) -> &str {
        &self.identity
    }

    pub const fn title(&self) -> &Text {
        &self.title
    }

    pub const fn summary(&self) -> &Text {
        &self.summary
    }

    pub fn figures(&self) -> &[Figure] {
        &self.figures
    }

    pub fn parts(&self) -> &[InsightPart] {
        &self.parts
    }

    pub fn notes(&self) -> &[Text] {
        &self.notes
    }

    /// Keeps only the parts for which `keep` holds, in their order.
    pub fn retain_parts(&mut self, keep: impl FnMut(&InsightPart) -> bool) {
        self.parts.retain(keep);
    }
}

#[cfg(test)]
mod tests {
    use crate::{ByteSize, DangerLevel, Figure, InsightPart, ShownCommand, Text, TextKey, ToolId};

    fn docker() -> ToolId {
        ToolId::new("docker").unwrap()
    }

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_owned()).collect()
    }

    fn text(key: &str) -> Text {
        Text::new(TextKey::new(key).unwrap())
    }

    fn figure() -> Figure {
        Figure { label: text("demo.figure"), size: ByteSize::new(4096) }
    }

    fn part(command: Option<ShownCommand>) -> InsightPart {
        InsightPart {
            label: text("demo.part.label"),
            size: ByteSize::new(2048),
            detail: text("demo.part.detail"),
            effect: text("demo.part.effect"),
            danger: DangerLevel::Safe,
            command,
        }
    }

    mod negative {
        use super::{args, docker, figure, part, text};
        use crate::{Insight, InsightError, MAX_FIGURES, MAX_INSIGHT_PARTS, MAX_NOTES, ShownCommand};

        #[test]
        fn a_command_without_arguments_is_refused() {
            assert_eq!(ShownCommand::new(docker(), vec![]), Err(InsightError::NoArguments));
        }

        #[test]
        fn an_empty_argument_is_refused() {
            assert_eq!(
                ShownCommand::new(docker(), args(&["image", ""])),
                Err(InsightError::NeedsQuoting { arg: String::new() })
            );
        }

        #[test]
        fn an_argument_that_needs_quoting_is_refused() {
            for arg in ["a b", "a;b", "$HOME", "\"x\""] {
                assert_eq!(
                    ShownCommand::new(docker(), args(&["image", arg])),
                    Err(InsightError::NeedsQuoting { arg: arg.into() }),
                    "{arg}"
                );
            }
        }

        #[test]
        fn every_form_of_force_is_refused() {
            for arg in ["-f", "--force", "--force=true", "-af"] {
                assert_eq!(
                    ShownCommand::new(docker(), args(&["image", "prune", arg])),
                    Err(InsightError::Forced { arg: arg.into() }),
                    "{arg}"
                );
            }
        }

        #[test]
        fn an_identity_not_in_kebab_case_is_refused() {
            for identity in ["", "Docker", "docker_desktop", "docker-", "a b"] {
                assert_eq!(
                    Insight::new(identity, text("demo.title"), text("demo.summary"), vec![], vec![], vec![]),
                    Err(InsightError::Identity { identity: identity.into() }),
                    "{identity}"
                );
            }
        }

        #[test]
        fn too_many_figures_are_refused() {
            let figures = vec![figure(); MAX_FIGURES + 1];

            assert_eq!(
                Insight::new("demo", text("demo.title"), text("demo.summary"), figures, vec![], vec![]),
                Err(InsightError::TooManyFigures { count: MAX_FIGURES + 1 })
            );
        }

        #[test]
        fn too_many_parts_are_refused() {
            let parts = vec![part(None); MAX_INSIGHT_PARTS + 1];

            assert_eq!(
                Insight::new("demo", text("demo.title"), text("demo.summary"), vec![], parts, vec![]),
                Err(InsightError::TooManyParts { count: MAX_INSIGHT_PARTS + 1 })
            );
        }

        #[test]
        fn too_many_notes_are_refused() {
            let notes = vec![text("demo.note"); MAX_NOTES + 1];

            assert_eq!(
                Insight::new("demo", text("demo.title"), text("demo.summary"), vec![], vec![], notes),
                Err(InsightError::TooManyNotes { count: MAX_NOTES + 1 })
            );
        }
    }

    mod positive {
        use serde_json::json;

        use super::{args, docker, figure, part, text};
        use crate::{Insight, MAX_FIGURES, MAX_INSIGHT_PARTS, MAX_NOTES, ShownCommand};

        #[test]
        fn the_line_joins_the_tool_and_its_arguments() {
            let command =
                ShownCommand::new(docker(), args(&["--context", "orbstack", "builder", "prune", "--all"])).unwrap();

            assert_eq!(command.line(), "docker --context orbstack builder prune --all");
            assert_eq!(command.tool(), docker());
            assert_eq!(command.args(), args(&["--context", "orbstack", "builder", "prune", "--all"]));
        }

        #[test]
        fn an_insight_at_every_maximum_is_accepted() {
            let insight = Insight::new(
                "docker-desktop",
                text("demo.title"),
                text("demo.summary"),
                vec![figure(); MAX_FIGURES],
                vec![part(None); MAX_INSIGHT_PARTS],
                vec![text("demo.note"); MAX_NOTES],
            )
            .unwrap();

            assert_eq!(insight.identity(), "docker-desktop");
            assert_eq!(insight.title(), &text("demo.title"));
            assert_eq!(insight.summary(), &text("demo.summary"));
            assert_eq!(insight.figures().len(), MAX_FIGURES);
            assert_eq!(insight.parts().len(), MAX_INSIGHT_PARTS);
            assert_eq!(insight.notes().len(), MAX_NOTES);
        }

        #[test]
        fn the_json_shape_is_camel_case_with_the_line() {
            let command = ShownCommand::new(docker(), args(&["volume", "prune"])).unwrap();
            let insight = Insight::new(
                "orbstack",
                text("demo.title"),
                text("demo.summary"),
                vec![figure()],
                vec![part(Some(command))],
                vec![text("demo.note")],
            )
            .unwrap();

            assert_eq!(
                serde_json::to_value(insight).unwrap(),
                json!({
                    "identity": "orbstack",
                    "title": { "key": "demo.title", "args": {} },
                    "summary": { "key": "demo.summary", "args": {} },
                    "figures": [{ "label": { "key": "demo.figure", "args": {} }, "size": 4096 }],
                    "parts": [{
                        "label": { "key": "demo.part.label", "args": {} },
                        "size": 2048,
                        "detail": { "key": "demo.part.detail", "args": {} },
                        "effect": { "key": "demo.part.effect", "args": {} },
                        "danger": "safe",
                        "command": { "tool": "docker", "args": ["volume", "prune"], "line": "docker volume prune" },
                    }],
                    "notes": [{ "key": "demo.note", "args": {} }],
                })
            );
        }
    }
}
