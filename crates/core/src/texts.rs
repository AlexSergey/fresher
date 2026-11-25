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
        AGREEMENT_LINES, AGREEMENT_VERSION, Access, AppDetail, AppOffer, AppsView, BlockedReason, ByteSize, Capability,
        CapabilityState, CapabilityText, CategoryText, CommandError, Completeness, DangerLevel, Ended, FootprintKind,
        ItemKind, KeptReason, KnownFolder, LastUse, Manager, MapPart, ModuleGroup, ModuleOutcome, MountKind,
        NotOffered, NotScanned, PartKind, Place, PlaceName, RemainderReason, Removability, SandboxTrashAt, ScanOutcome,
        SettingGroup, SettingsSchema, SkipCode, SkipReason, SummaryKind, Text, TextArg, TextKey, ToolId, TrashState,
        Unchanged, UninstallChoice, VolumeMap, Withheld,
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
            ScanOutcome::Stopped => "stopped",
            ScanOutcome::Failed { .. } => "failed",
        }
    }

    fn kebab_name(value: impl serde::Serialize) -> String {
        serde_json::to_value(value).unwrap().as_str().unwrap().to_owned()
    }

    /// The danger levels' names, meanings and zones.
    fn danger_keys() -> Vec<TextKey> {
        let key = |key: String| TextKey::new(key).unwrap();

        DangerLevel::ALL
            .into_iter()
            .flat_map(|level| {
                let level = kebab_name(level);
                ["name", "meaning", "zone"].map(|text| key(format!("core.danger.{level}.{text}")))
            })
            .collect()
    }

    /// Every key the core can emit, built from the enums, the error codes and the core's settings.
    fn core_keys() -> BTreeSet<TextKey> {
        let key = |key: String| TextKey::new(key).unwrap();
        let danger = danger_keys().into_iter();
        let groups = ModuleGroup::ALL.into_iter().flat_map(|group| [group.name_key(), group.description_key()]);
        let completeness = [Completeness::Complete, Completeness::Partial, Completeness::Unknown]
            .map(|completeness| key(format!("core.completeness.{}", completeness_segment(completeness))));
        let errors = CommandError::CODES.map(|code| key(format!("core.error.{code}")));
        let failed = ScanOutcome::Failed { reason: Text::new(key("core.error.internal".into())) };
        let outcomes = [ScanOutcome::Completed, ScanOutcome::Partial, ScanOutcome::Stopped, failed]
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
        let settings = schema
            .descriptors()
            .iter()
            .filter(|setting| setting.key().namespace() == "core" && setting.group() != SettingGroup::Agreement)
            .flat_map(|setting| {
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
        let storage = storage_keys();
        let largest = ItemKind::ALL
            .map(ItemKind::text_key)
            .into_iter()
            .chain(Withheld::ALL.map(Withheld::text_key))
            .chain(Unchanged::ALL.map(Unchanged::text_key))
            .chain(own_file_keys())
            .chain([unpacked().key().clone()]);
        let start = std::iter::once(key("core.start.failed".into()));
        let trash = [sandbox_trash(), SandboxTrashAt::System.text()].map(|text| text.key().clone());
        let left_out = left_out_summaries().into_iter().map(|text| text.key().clone());
        let apps = apps_keys();
        let agreement = ["title", "intro"]
            .map(|name| key(format!("core.agreement.{name}")))
            .into_iter()
            .chain(AGREEMENT_LINES.map(|id| key(format!("core.agreement.line.{id}"))))
            // What changed in each version after the first: `v2` up to the current one, none while it is 1.
            .chain(
                (2..)
                    .take_while(|version| *version <= AGREEMENT_VERSION)
                    .map(|version| key(format!("core.agreement.changes.v{version}"))),
            );
        let cleanup_states = Ended::ALL
            .map(|ended| ended.text().key().clone())
            .into_iter()
            .chain(TrashState::ALL.map(|state| state.text().key().clone()))
            .chain(BlockedReason::ALL.map(|reason| reason.text().key().clone()));

        danger
            .chain(groups)
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
            .chain(storage)
            .chain(start)
            .chain(trash)
            .chain(left_out)
            .chain(settings)
            .chain(largest)
            .chain(apps)
            .chain(agreement)
            .collect()
    }

    /// The Apps screen's and the uninstall's keys: the two categories' texts, the parts, the choices, the kept and
    /// not-offered reasons, the managers, the list's and the page's notices, and the texts with args (`apps_texts`).
    fn apps_keys() -> Vec<TextKey> {
        let categories = crate::uninstall_categories()
            .into_iter()
            .flat_map(|category| CategoryText::ALL.map(|text| category.text_key(text)));
        let parts = FootprintKind::ALL.map(FootprintKind::name_key);
        let choices =
            UninstallChoice::ALL.into_iter().flat_map(|choice| [choice.label_key(), choice.description_key()]);
        let kept = KeptReason::ALL.map(|reason| reason.text().key().clone());
        let not_offered = NotOffered::ALL.map(NotOffered::text_key);
        let notices = [AppsView::unlisted(), AppsView::listed(Vec::new())].into_iter().filter_map(|view| view.notice);
        let management = Access::ALL.into_iter().filter_map(AppDetail::management_notice);
        categories
            .chain(parts)
            .chain(choices)
            .chain(kept)
            .chain(not_offered)
            .chain(notices.map(|text| text.key().clone()))
            .chain(management.map(|text| text.key().clone()))
            .chain(apps_texts().iter().map(|text| text.key().clone()))
            .collect()
    }

    /// The Apps screen's and the uninstall's texts with args, as the core builds them.
    fn apps_texts() -> Vec<Text> {
        let size = ByteSize::new(4096);
        let since = crate::Timestamp::from_secs(1);
        let summaries = [LastUse::Unused { since }, LastUse::Used, LastUse::Unknown]
            .map(|last_use| last_use.summary("Editor", size));
        let reasons = UninstallChoice::ALL.map(|choice| {
            Text::new(choice.reason_key()).with("app", TextArg::Text("Editor".into())).with("size", TextArg::Size(size))
        });
        let installed_by = match AppOffer::installed_by(Manager::Homebrew) {
            AppOffer::NotOffered { text, .. } => Some(text),
            AppOffer::Offered => None,
        };
        summaries.into_iter().chain(reasons).chain(installed_by).chain([crate::uninstall_title("Editor")]).collect()
    }

    /// The own-file category's six texts, the title of a picked item and the two reasons.
    fn own_file_keys() -> Vec<TextKey> {
        let category = crate::own_file();
        let texts = CategoryText::ALL.map(|text| category.text_key(text));
        let others =
            ["title", "reason.file", "reason.folder"].map(|name| TextKey::new(format!("core.own-file.{name}")));
        texts.into_iter().chain(others.into_iter().map(Result::unwrap)).collect()
    }

    /// The line under an archive's row, with the arg the engine passes.
    fn unpacked() -> Text {
        Text::new(TextKey::new("core.largest.unpacked").unwrap()).with("folder", TextArg::Text("Movie".into()))
    }

    /// The title of a picked item, with the arg the engine passes.
    fn own_title() -> Text {
        crate::own_file_title("Movie.mov")
    }

    /// The storage map's keys: every part's and summary part's name and description, the remainder's names, its
    /// reasons, the named places, the volume kinds, the volume's notes and why a folder of the place view was not
    /// scanned.
    fn storage_keys() -> Vec<TextKey> {
        let remainders = [true, false].map(|outside_scan| PartKind::Remainder { outside_scan, reasons: vec![] });
        let parts = crate::storage::every_kind()
            .into_iter()
            .chain(remainders)
            .flat_map(|kind| [kind.name_key(), kind.description_key()]);
        let summary = SummaryKind::ALL.into_iter().flat_map(|kind| [kind.name_key(), kind.description_key()]);
        let reasons = crate::storage::every_reason().into_iter().map(RemainderReason::text_key);
        let trash = [PlaceName::Trash.name_key(), PlaceName::Trash.description_key()].into_iter().flatten();
        let kinds = [MountKind::Internal, MountKind::External, MountKind::Network].map(MountKind::text_key);
        let notes = volume_notes().changed_note().into_iter().chain(purgeable_note().map(|note| note.key().clone()));
        let not_scanned = NotScanned::ALL.map(NotScanned::text_key);
        let walked = KnownFolder::ALL
            .into_iter()
            .filter(|folder| folder.is_walked_place())
            .map(|folder| Place { folder, path: std::path::PathBuf::new() }.name_key());
        parts
            .chain(summary)
            .chain(reasons)
            .chain(trash)
            .chain(kinds)
            .chain(notes)
            .chain(not_scanned)
            .chain(walked)
            .collect()
    }

    fn volume_notes() -> VolumeMap {
        let free = MapPart { kind: PartKind::Free, sizes: crate::Sizes::ZERO };
        VolumeMap::new(
            "Disk".into(),
            MountKind::Internal,
            ByteSize::ZERO,
            vec![free],
            Some(ByteSize::new(1)),
            vec![],
            true,
        )
        .unwrap()
    }

    fn purgeable_note() -> Option<Text> {
        volume_notes().purgeable_note()
    }

    /// The lines over a category's left-outs, held back for now and kept for good, both plural forms, with the args
    /// the UI passes to `core.left-out.summary` and `core.left-out.kept`: its translator picks the form by `count`.
    fn left_out_summaries() -> [Text; 4] {
        let size = ByteSize::new(1024);
        [
            ("core.left-out.summary.one", 1),
            ("core.left-out.summary.other", 2),
            ("core.left-out.kept.one", 1),
            ("core.left-out.kept.other", 2),
        ]
        .map(|(form, count)| {
            Text::new(TextKey::new(form).unwrap())
                .with("count", TextArg::Count(count))
                .with("size", TextArg::Size(size))
        })
    }

    /// Where a sandbox's cleanups go when they stay in it: its own Trash folder.
    fn sandbox_trash() -> Text {
        SandboxTrashAt::Sandbox(std::path::Path::new("/s/home/.Trash")).text()
    }

    /// The texts the core builds with arguments: the errors, the tool notes, the skips' instructions, the failures,
    /// the purgeable note, the left-outs' summaries and the sandbox's own Trash.
    fn texts_with_args() -> Vec<Text> {
        let errors = one_error_of_each().iter().map(CommandError::text).collect::<Vec<_>>();
        let skips = outcome_sample::skip_reasons().iter().filter_map(SkipReason::how).collect::<Vec<_>>();
        let failures = outcome_sample::failures().iter().map(crate::Failure::text).collect::<Vec<_>>();
        errors
            .into_iter()
            .chain(tool_notes())
            .chain(skips)
            .chain(failures)
            .chain(purgeable_note())
            .chain(left_out_summaries())
            .chain([sandbox_trash(), unpacked(), own_title()])
            .chain(apps_texts())
            .collect()
    }

    /// The text as a reader sees it: its template with every placeholder it names filled in.
    fn render(texts: &crate::Dictionary, text: &Text) -> String {
        let template = texts.get(text.key().as_str()).unwrap().to_owned();
        text.args().iter().fold(template, |rendered, (name, arg)| {
            let value = match arg {
                TextArg::Text(value) => value.clone(),
                other => format!("{other:?}"),
            };
            rendered.replace(&format!("{{{name}}}"), &value)
        })
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

        use super::{core_keys, render, texts_with_args};
        use crate::{Capability, CapabilityText, DETAIL_ARG, Failure, PartKind, TextKey, core_dictionary};

        #[test]
        fn no_core_template_names_the_detail_arg() {
            let texts = core_dictionary().unwrap();

            for key in texts.keys() {
                let placeholders = texts.placeholders(key.as_str()).unwrap();
                assert!(!placeholders.contains(DETAIL_ARG), "{key} names {{{DETAIL_ARG}}}");
            }
        }

        #[test]
        fn a_tools_failure_sentence_holds_no_tool_output() {
            let texts = core_dictionary().unwrap();
            let line = "Error: conflict: unable to remove repository reference";

            let with_line = render(&texts, &Failure::ToolFailed { detail: line.into() }.text());
            let without = render(&texts, &Failure::ToolFailed { detail: String::new() }.text());

            assert_eq!(with_line, without);
            assert!(!with_line.contains(line), "{with_line}");
        }

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
        fn every_part_kind_has_a_name_and_a_description() {
            let texts = core_dictionary().unwrap();
            let remainders = [true, false].map(|outside_scan| PartKind::Remainder { outside_scan, reasons: vec![] });

            for kind in crate::storage::every_kind().into_iter().chain(remainders) {
                for key in [kind.name_key(), kind.description_key()] {
                    assert!(texts.get(key.as_str()).is_some_and(|text| !text.is_empty()), "{key}");
                }
            }
        }

        #[test]
        fn no_text_says_other() {
            let texts = core_dictionary().unwrap();

            for key in texts.keys().filter(|key| key.as_str().starts_with("core.storage.")) {
                let text = texts.get(key.as_str()).unwrap();
                for word in ["Other", "Miscellaneous", "System Data"] {
                    assert!(!text.contains(word), "{key}: {text}");
                }
            }
        }

        #[test]
        fn no_text_mentions_sudo() {
            let texts = core_dictionary().unwrap();

            for key in texts.keys() {
                let text = texts.get(key.as_str()).unwrap();
                assert!(!text.to_lowercase().contains("sudo"), "{key}: {text}");
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
        fn every_group_has_a_name_and_a_description() {
            let texts = core_dictionary().unwrap();

            for group in crate::ModuleGroup::ALL {
                for key in [group.name_key(), group.description_key()] {
                    assert!(texts.get(key.as_str()).is_some_and(|text| !text.is_empty()), "{key}");
                }
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
                let args: BTreeSet<&str> =
                    text.args().keys().map(String::as_str).filter(|name| *name != DETAIL_ARG).collect();
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
        fn the_start_text_is_in_the_core_dictionary() {
            let texts = core_dictionary().unwrap();

            assert!(core_keys().iter().any(|key| key.as_str() == "core.start.failed"));
            assert!(texts.get("core.start.failed").is_some());
        }

        #[test]
        fn the_summary_text_is_in_the_core_dictionary() {
            let texts = core_dictionary().unwrap();

            for form in ["core.left-out.summary.one", "core.left-out.summary.other"] {
                assert!(core_keys().iter().any(|key| key.as_str() == form), "{form}");
                assert!(texts.get(form).is_some_and(|text| !text.is_empty()), "{form}");
            }
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

        #[test]
        fn names_every_zone() {
            let texts = core_dictionary().unwrap();

            let zones: Vec<_> = DangerLevel::ALL
                .into_iter()
                .map(|level| texts.get(&format!("core.danger.{}.zone", super::kebab_name(level))))
                .collect();

            assert_eq!(
                zones,
                [
                    Some("Safe to remove"),
                    Some("Usually safe"),
                    Some("Look before removing"),
                    Some("Only if you are sure")
                ]
            );
        }
    }
}
