//! The core's own texts: danger levels, size completeness, command errors, the core's settings and scan outcomes.

use crate::{Dictionary, DictionaryError};

/// The core's dictionary, the `core.` namespace, embedded from `locales/en.json`. The engine merges it in front of the
/// modules' dictionaries.
pub fn core_dictionary() -> Result<Dictionary, DictionaryError> {
    Dictionary::parse("core", include_str!("../locales/en.json"))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use crate::{
        CommandError, Completeness, DangerLevel, FindingId, ModuleOutcome, Operation, ScanOutcome, SettingKey,
        SettingsSchema, Text, TextKey,
    };

    /// One error of every variant; the `match` has no wildcard, so a new variant fails to compile until it is here.
    fn one_error_of_each() -> Vec<CommandError> {
        let sample = |error: CommandError| match error {
            CommandError::Busy { .. }
            | CommandError::NotRunning
            | CommandError::UnknownFinding { .. }
            | CommandError::UnknownModule { .. }
            | CommandError::UnknownSetting { .. }
            | CommandError::InvalidSettingValue { .. }
            | CommandError::UnknownLocale { .. }
            | CommandError::SettingsReadOnly
            | CommandError::Storage
            | CommandError::NoScan
            | CommandError::StaleNode
            | CommandError::Internal => error,
        };
        vec![
            sample(CommandError::Busy { running: Operation::Scan }),
            sample(CommandError::NotRunning),
            sample(CommandError::UnknownFinding { id: FindingId::new(9) }),
            sample(CommandError::UnknownModule { id: "gone".into() }),
            sample(CommandError::UnknownSetting { key: "demo.gone".into() }),
            sample(CommandError::InvalidSettingValue { key: SettingKey::new("demo.deep").unwrap() }),
            sample(CommandError::UnknownLocale { locale: "fr".into() }),
            sample(CommandError::SettingsReadOnly),
            sample(CommandError::Storage),
            sample(CommandError::NoScan),
            sample(CommandError::StaleNode),
            sample(CommandError::Internal),
        ]
    }

    fn completeness_segment(completeness: Completeness) -> &'static str {
        match completeness {
            Completeness::Complete => "complete",
            Completeness::Partial => "partial",
            Completeness::Unknown => "unknown",
        }
    }

    fn outcome_segment(outcome: &ScanOutcome) -> &'static str {
        match outcome {
            ScanOutcome::Completed => "completed",
            ScanOutcome::Partial => "partial",
            ScanOutcome::Failed { .. } => "failed",
        }
    }

    fn kebab_name(value: impl serde::Serialize) -> String {
        serde_json::to_value(value).unwrap().as_str().unwrap().to_owned()
    }

    /// Every key the core can emit, built from the enums, the error codes and the core's settings.
    fn core_keys() -> BTreeSet<TextKey> {
        let key = |key: String| TextKey::new(key).unwrap();
        let danger = DangerLevel::ALL.into_iter().flat_map(|level| {
            let level = kebab_name(level);
            [key(format!("core.danger.{level}.name")), key(format!("core.danger.{level}.meaning"))]
        });
        let completeness = [Completeness::Complete, Completeness::Partial, Completeness::Unknown]
            .map(|completeness| key(format!("core.completeness.{}", completeness_segment(completeness))));
        let errors = CommandError::CODES.map(|code| key(format!("core.error.{code}")));
        let failed = ScanOutcome::Failed { reason: Text::new(key("core.error.internal".into())) };
        let outcomes = [ScanOutcome::Completed, ScanOutcome::Partial, failed]
            .map(|outcome| key(format!("core.scan.outcome.{}", outcome_segment(&outcome))));
        let panicked = match ModuleOutcome::panicked() {
            ModuleOutcome::Failed { reason } => reason.key().clone(),
            ModuleOutcome::Completed | ModuleOutcome::Cancelled => key("core.error.internal".into()),
        };
        let schema = SettingsSchema::new([]).unwrap();
        let settings = schema
            .descriptors()
            .iter()
            .filter(|setting| setting.key().namespace() == "core")
            .flat_map(|setting| [setting.key().label_key(), setting.key().description_key()]);

        danger.chain(completeness).chain(errors).chain(outcomes).chain([panicked]).chain(settings).collect()
    }

    mod negative {
        use std::collections::BTreeSet;

        use super::{core_keys, one_error_of_each};
        use crate::{TextKey, core_dictionary};

        #[test]
        fn every_key_the_core_can_emit_exists() {
            let texts = core_dictionary().unwrap();

            let missing: Vec<_> = core_keys().into_iter().filter(|key| texts.get(key.as_str()).is_none()).collect();

            assert!(missing.is_empty(), "missing in crates/core/locales/en.json: {missing:?}");
        }

        #[test]
        fn no_key_in_the_file_is_unused() {
            let emitted = core_keys();
            let texts = core_dictionary().unwrap();

            let unused: Vec<&TextKey> = texts.keys().filter(|key| !emitted.contains(*key)).collect();

            assert!(unused.is_empty(), "not emitted by the core: {unused:?}");
        }

        #[test]
        fn every_placeholder_matches_the_args_the_core_passes() {
            let texts = core_dictionary().unwrap();

            for error in one_error_of_each() {
                let text = error.text();
                let args: BTreeSet<&str> = text.args().keys().map(String::as_str).collect();
                assert_eq!(texts.placeholders(text.key().as_str()), Some(args), "{}", text.key());
            }
        }

        #[test]
        fn the_other_texts_have_no_placeholders() {
            let texts = core_dictionary().unwrap();

            for key in core_keys().iter().filter(|key| !key.as_str().starts_with("core.error.")) {
                assert_eq!(texts.placeholders(key.as_str()), Some(BTreeSet::new()), "{key}");
            }
        }
    }

    mod positive {
        use super::core_keys;
        use crate::{DangerLevel, core_dictionary};

        #[test]
        fn the_core_dictionary_parses() {
            let texts = core_dictionary().unwrap();

            assert_eq!(texts.len(), core_keys().len());
            assert!(texts.keys().all(|key| key.namespace() == "core"));
        }

        #[test]
        fn names_every_danger_level() {
            let texts = core_dictionary().unwrap();

            let names: Vec<_> = DangerLevel::ALL
                .into_iter()
                .map(|level| texts.get(&format!("core.danger.{}.name", super::kebab_name(level))))
                .collect();

            assert_eq!(names, [Some("Safe"), Some("Low"), Some("Medium"), Some("High")]);
        }
    }
}
