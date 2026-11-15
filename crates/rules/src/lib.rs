//! The shared matcher for declarative rule data; holds no rules itself.
//!
//! A module keeps its rules in `rules.toml` next to its code and embeds them with `include_str!`; [`RuleSet::parse`]
//! validates them against the module's descriptor, [`RuleSet::interests`] tells the walk where to look, and
//! [`RuleSet::evaluate`] turns what the walk matched into findings, with no IO but one bounded read of a
//! `CACHEDIR.TAG`.

mod cachedir;
mod compile;
mod evaluate;
mod format;
mod glob;
mod rules;
mod version;

pub use rules::{Rule, RuleError, RuleSet};
