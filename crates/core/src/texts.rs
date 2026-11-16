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
        Access, BlockedReason, Capability, CapabilityState, CapabilityText, CommandError, Completeness, DangerLevel,
        Ended, ModuleOutcome, Removability, ScanOutcome, SettingsSchema, SkipCode, SkipReason, Text, TextKey, ToolId,
        TrashState,
    };

    use crate::cleanup::outcome_sample;
    use crate::error::tests::one_of_each as one_error_of_each;

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
            ModuleOutcome::Completed | ModuleOutcome::Cancelled | ModuleOutcome::Unavailable { .. } => {
                key("core.error.internal".into())
            }
        };
        let module = [panicked, ModuleOutcome::unavailable_key()];
        let capabilities = Capability::SYSTEM
            .into_iter()
            .flat_map(|capability| CapabilityText::ALL.into_iter().filter_map(move |text| capability.text_key(text)));
        let states = Access::ALL.map(Access::text_key);
        let removability =
            Removability::ALL.into_iter().flat_map(|state| std::iter::once(state.name_key()).chain(state.how_key()));
        let schema = SettingsSchema::new([]).unwrap();
        let settings =
            schema.descriptors().iter().filter(|setting| setting.key().namespace() == "core").flat_map(|setting| {
                let options = match setting.kind() {
                    crate::SettingKind::Choice { options } => options.clone(),
                    _ => Vec::new(),
                };
                let options = options.into_iter().map(|option| key(format!("{}.options.{option}", setting.key())));
                [setting.key().label_key(), setting.key().description_key()].into_iter().chain(options)
            });

        let tools = tool_notes().into_iter().map(|note| note.key().clone());
        let skips = SkipCode::ALL.into_iter().flat_map(|code| std::iter::once(code.text_key()).chain(code.how_key()));
        let item_outcomes = outcome_sample::item_outcomes().into_iter().map(|outcome| outcome.text().key().clone());
        let recorded = outcome_sample::recorded_outcomes().into_iter().map(|outcome| outcome.text().key().clone());
        let failures = outcome_sample::failures().into_iter().map(|failure| failure.text().key().clone());
        let put_backs = outcome_sample::put_back_outcomes().into_iter().map(|outcome| outcome.text().key().clone());
        let cleanup_states = Ended::ALL
            .map(|ended| ended.text().key().clone())
            .into_iter()
            .chain(TrashState::ALL.map(|state| state.text().key().clone()))
            .chain(BlockedReason::ALL.map(|reason| reason.text().key().clone()));

        danger
            .chain(completeness)
            .chain(errors)
            .chain(outcomes)
            .chain(module)
            .chain(capabilities)
            .chain(states)
            .chain(removability)
            .chain(tools)
            .chain(skips)
            .chain(item_outcomes)
            .chain(recorded)
            .chain(failures)
            .chain(put_backs)
            .chain(cleanup_states)
            .chain(settings)
            .collect()
    }

    /// The texts the core builds with arguments: the errors, the tool notes, the skips' instructions, the failures.
    fn texts_with_args() -> Vec<Text> {
        let errors = one_error_of_each().iter().map(CommandError::text).collect::<Vec<_>>();
        let skips = outcome_sample::skip_reasons().iter().filter_map(SkipReason::how).collect::<Vec<_>>();
        let failures = outcome_sample::failures().iter().map(crate::Failure::text).collect::<Vec<_>>();
        errors.into_iter().chain(tool_notes()).chain(skips).chain(failures).collect()
    }

    /// The notes of a tool's state, with the arguments the core passes.
    fn tool_notes() -> Vec<Text> {
        let tool = ToolId::new("docker").unwrap();
        [CapabilityState::tool_not_installed(tool), CapabilityState::tool_not_ready(tool)]
            .into_iter()
            .filter_map(|state| state.note)
            .collect()
    }

    mod negative {
        use std::collections::BTreeSet;

        use super::{core_keys, texts_with_args};
        use crate::{Capability, CapabilityText, TextKey, core_dictionary};

        #[test]
        fn every_skip_reason_has_its_text() {
            let texts = core_dictionary().unwrap();

            for reason in crate::cleanup::outcome_sample::skip_reasons() {
                assert!(texts.get(reason.text().key().as_str()).is_some(), "{reason:?}");
                if let Some(how) = reason.how() {
                    assert!(texts.get(how.key().as_str()).is_some(), "{reason:?}");
                }
            }
        }

        #[test]
        fn every_new_code_has_its_text() {
            let texts = core_dictionary().unwrap();

            for code in crate::CommandError::CODES {
                assert!(texts.get(&format!("core.error.{code}")).is_some(), "{code}");
            }
        }

        #[test]
        fn every_capability_has_its_texts() {
            let texts = core_dictionary().unwrap();

            for capability in Capability::SYSTEM {
                for text in CapabilityText::ALL {
                    let key = capability.text_key(text).unwrap();
                    assert!(texts.get(key.as_str()).is_some_and(|text| !text.is_empty()), "{key}");
                }
            }
        }

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

            for text in texts_with_args() {
                let args: BTreeSet<&str> = text.args().keys().map(String::as_str).collect();
                assert_eq!(texts.placeholders(text.key().as_str()), Some(args), "{}", text.key());
            }
        }

        #[test]
        fn the_other_texts_have_no_placeholders() {
            let texts = core_dictionary().unwrap();

            let with_args: BTreeSet<TextKey> = texts_with_args().iter().map(|text| text.key().clone()).collect();
            for key in core_keys().iter().filter(|key| !with_args.contains(*key)) {
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
