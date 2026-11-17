//! The TypeScript types of the IPC, generated from the core's Rust types into `app/ui/src/ipc/bindings.generated.ts`.
//!
//! The file is the output of `ts-rs` as it is: one declaration per type, in the order of `declarations()`, each with
//! its doc comment. `Prettier` and `ESLint` skip `*.generated.ts`, so the test compares the file byte for byte, with no
//! formatter in between. `UPDATE_BINDINGS=1 cargo nextest run -p fresher-app --test bindings` rewrites it; the diff
//! is reviewed before commit, like a golden file.

// Helpers of an integration test are test code too; `cfg(test)` lets Clippy's in-test allowances apply to them.
#[cfg(test)]
mod support {
    use std::path::PathBuf;

    use fresher_core::{
        Access, AccessStatus, ActionKind, AppInfo, Availability, BlockedReason, Build, ByteSize, Capability,
        CapabilityState, CategoryDescriptor, CategoryId, CategoryInfo, CategoryPick, CategoryTotal, ChildOrder,
        CleanupDetail, CleanupId, CleanupItemRef, CleanupItemView, CleanupPlanView, CleanupReport, CleanupSelection,
        CleanupSummary, CleanupsPage, CleanupsQuery, CommandError, Completeness, Confirmation, Coverage, DangerLevel,
        Dictionary, Ended, EngineStatus, Event, Finding, FindingId, FindingKey, FindingOrder, FindingsPage,
        FindingsQuery, GuardedFolder, HiddenBy, HiddenModule, IdOf, IgnoreTarget, IgnoredCount, IgnoredEntry,
        ItemOutcome, ItemReport, LastScan, LeftOut, LeftOutCount, Locale, Method, ModuleGroup, ModuleId, ModuleInfo,
        ModuleOutcome, Need, NeedKind, NodeId, NodeInfo, NodeKind, NodePage, NotEntered, NotScanned, Operation,
        OutcomeCounts, Overview, OwnData, PlanGroup, PlanId, PlanItemView, PlanSummary, PutBackItem, PutBackOutcome,
        PutBackReport, PutBackRequest, RecordedOutcome, Refusal, Removability, Removal, ScanId, ScanKind, ScanOutcome,
        SettingDescriptor, SettingEffect, SettingGroup, SettingKey, SettingKind, SettingProblem, SettingUpdate,
        SettingValue, SettingsFile, SettingsView, Sizes, SkipCode, SkipReason, StateId, Subject, Text, TextArg,
        TextKey, Timestamp, ToolCommand, ToolId, TrashState, Unit, Unreadable, UnreadableCount, UnreadablePlace,
        VolumeKind,
    };
    use fresher_core::{
        AppRef, CategoryShare, Descent, MapPart, MapPhase, MeaningGroup, MountKind, PartKind, PlaceName, PlaceRow,
        PlaceShare, PlaceView, RemainderReason, StorageMap, VolumeMap,
    };
    use ts_rs::{Config, TS};

    /// One exported type: its TypeScript name, its declaration and the names of the types it refers to.
    pub(crate) struct Declaration {
        pub(crate) name: String,
        pub(crate) text: String,
        pub(crate) dependencies: Vec<String>,
    }

    fn declaration<T: TS + 'static>(config: &Config) -> Declaration {
        let text = format!("{}export {}", T::docs().unwrap_or_default(), T::decl(config));
        let dependencies = T::dependencies(config).into_iter().map(|dependency| dependency.ts_name).collect();
        Declaration { name: T::name(config), text, dependencies }
    }

    /// Every type the IPC carries: the events, the commands' inputs and outputs, the refusal, and all they contain; the
    /// disk state's and the cleanups' types too, before a command returns them.
    pub(crate) fn declarations() -> Vec<Declaration> {
        let config = Config::new();
        let mut declarations = vec![
            declaration::<AppInfo>(&config),
            declaration::<Build>(&config),
            declaration::<Event>(&config),
            declaration::<Refusal>(&config),
            declaration::<CommandError>(&config),
            declaration::<Overview>(&config),
            declaration::<EngineStatus>(&config),
            declaration::<Operation>(&config),
            declaration::<LastScan>(&config),
            declaration::<IgnoredCount>(&config),
            declaration::<HiddenModule>(&config),
            declaration::<HiddenBy>(&config),
            declaration::<IgnoreTarget>(&config),
            declaration::<IgnoredEntry>(&config),
            declaration::<OwnData>(&config),
            declaration::<Coverage>(&config),
            declaration::<NotEntered>(&config),
            declaration::<UnreadableCount>(&config),
            declaration::<CategoryTotal>(&config),
            declaration::<ModuleGroup>(&config),
            declaration::<ModuleInfo>(&config),
            declaration::<CategoryInfo>(&config),
            declaration::<Availability>(&config),
            declaration::<FindingsQuery>(&config),
            declaration::<FindingOrder>(&config),
            declaration::<FindingsPage>(&config),
            declaration::<Finding>(&config),
            declaration::<FindingId>(&config),
            declaration::<FindingKey>(&config),
            declaration::<Subject>(&config),
            declaration::<Removability>(&config),
            declaration::<Removal>(&config),
            declaration::<ToolCommand>(&config),
            declaration::<ToolId>(&config),
            declaration::<CategoryDescriptor>(&config),
            declaration::<CategoryId>(&config),
            declaration::<ModuleId>(&config),
            declaration::<ScanId>(&config),
            declaration::<ModuleOutcome>(&config),
            declaration::<LeftOutCount>(&config),
            declaration::<ScanOutcome>(&config),
            declaration::<ScanKind>(&config),
            declaration::<DangerLevel>(&config),
            declaration::<Sizes>(&config),
            declaration::<ByteSize>(&config),
            declaration::<Completeness>(&config),
            declaration::<Timestamp>(&config),
            declaration::<SettingsView>(&config),
            declaration::<SettingsFile>(&config),
            declaration::<SettingDescriptor>(&config),
            declaration::<SettingKey>(&config),
            declaration::<SettingKind>(&config),
            declaration::<IdOf>(&config),
            declaration::<SettingGroup>(&config),
            declaration::<SettingEffect>(&config),
            declaration::<Unit>(&config),
            declaration::<SettingValue>(&config),
            declaration::<SettingProblem>(&config),
            declaration::<SettingUpdate>(&config),
            declaration::<Locale>(&config),
            declaration::<Dictionary>(&config),
            declaration::<Text>(&config),
            declaration::<TextKey>(&config),
            declaration::<TextArg>(&config),
            declaration::<AccessStatus>(&config),
            declaration::<CapabilityState>(&config),
            declaration::<Capability>(&config),
            declaration::<GuardedFolder>(&config),
            declaration::<VolumeKind>(&config),
            declaration::<Access>(&config),
            declaration::<Need>(&config),
            declaration::<NeedKind>(&config),
            declaration::<NodeInfo>(&config),
            declaration::<NodeId>(&config),
            declaration::<StateId>(&config),
            declaration::<NodeKind>(&config),
            declaration::<NotScanned>(&config),
            declaration::<Unreadable>(&config),
            declaration::<ChildOrder>(&config),
            declaration::<NodePage>(&config),
            declaration::<UnreadablePlace>(&config),
        ];
        declarations.extend(storage_declarations(&config));
        declarations.extend(cleanup_declarations(&config));
        declarations
    }

    /// The types of the storage map's commands.
    fn storage_declarations(config: &Config) -> Vec<Declaration> {
        vec![
            declaration::<StorageMap>(config),
            declaration::<MapPhase>(config),
            declaration::<VolumeMap>(config),
            declaration::<MountKind>(config),
            declaration::<MapPart>(config),
            declaration::<PartKind>(config),
            declaration::<RemainderReason>(config),
            declaration::<MeaningGroup>(config),
            declaration::<AppRef>(config),
            declaration::<CategoryShare>(config),
            declaration::<PlaceShare>(config),
            declaration::<PlaceName>(config),
            declaration::<PlaceView>(config),
            declaration::<PlaceRow>(config),
            declaration::<Descent>(config),
        ]
    }

    /// The types of the cleanup commands and events.
    fn cleanup_declarations(config: &Config) -> Vec<Declaration> {
        vec![
            declaration::<CleanupSelection>(config),
            declaration::<CategoryPick>(config),
            declaration::<CleanupPlanView>(config),
            declaration::<PlanId>(config),
            declaration::<PlanSummary>(config),
            declaration::<PlanGroup>(config),
            declaration::<ActionKind>(config),
            declaration::<PlanItemView>(config),
            declaration::<LeftOut>(config),
            declaration::<SkipReason>(config),
            declaration::<Confirmation>(config),
            declaration::<Method>(config),
            declaration::<CleanupId>(config),
            declaration::<ItemReport>(config),
            declaration::<ItemOutcome>(config),
            declaration::<CleanupReport>(config),
            declaration::<Ended>(config),
            declaration::<OutcomeCounts>(config),
            declaration::<BlockedReason>(config),
            declaration::<CleanupsQuery>(config),
            declaration::<CleanupsPage>(config),
            declaration::<CleanupSummary>(config),
            declaration::<CleanupDetail>(config),
            declaration::<CleanupItemView>(config),
            declaration::<RecordedOutcome>(config),
            declaration::<SkipCode>(config),
            declaration::<TrashState>(config),
            declaration::<PutBackRequest>(config),
            declaration::<CleanupItemRef>(config),
            declaration::<PutBackReport>(config),
            declaration::<PutBackItem>(config),
            declaration::<PutBackOutcome>(config),
        ]
    }

    /// The whole file: the header, then every declaration, separated by a blank line.
    pub(crate) fn render(declarations: &[Declaration]) -> String {
        let header = "// Generated by `app/tauri/tests/bindings.rs` from the core's Rust types; do not edit.\n\
                      // `UPDATE_BINDINGS=1 cargo nextest run -p fresher-app --test bindings` rewrites it.\n";
        let body: Vec<&str> = declarations.iter().map(|declaration| declaration.text.as_str()).collect();
        format!("{header}\n{}\n", body.join("\n\n"))
    }

    pub(crate) fn bindings_file() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../ui/src/ipc/bindings.generated.ts")
    }
}

mod negative {
    use std::collections::BTreeSet;

    use crate::support::declarations;

    #[test]
    fn no_type_refers_to_a_type_left_out_of_the_file() {
        let declarations = declarations();
        let declared: BTreeSet<&str> = declarations.iter().map(|declaration| declaration.name.as_str()).collect();

        for declaration in &declarations {
            for dependency in &declaration.dependencies {
                assert!(declared.contains(dependency.as_str()), "{} refers to {dependency}", declaration.name);
            }
        }
        assert_eq!(declared.len(), declarations.len(), "a type is declared twice");
    }

    #[test]
    fn no_field_is_in_snake_case() {
        for declaration in declarations() {
            let code = declaration.text.lines().filter(|line| !line.trim_start().starts_with(['/', '*']));
            let snake: Vec<&str> =
                code.flat_map(str::split_whitespace).filter(|word| word.ends_with(':') && word.contains('_')).collect();
            assert!(snake.is_empty(), "{} has snake_case fields: {snake:?}", declaration.name);
        }
    }

    #[test]
    fn no_number_becomes_a_bigint() {
        for declaration in declarations() {
            assert!(!declaration.text.contains("bigint"), "{}", declaration.text);
        }
    }
}

mod positive {
    use crate::support::{bindings_file, declarations, render};

    #[test]
    fn the_bindings_file_is_up_to_date() {
        let actual = render(&declarations());
        let file = bindings_file();

        if std::env::var_os("UPDATE_BINDINGS").is_some() {
            std::fs::write(&file, actual).unwrap();
            return;
        }

        let expected = std::fs::read_to_string(&file).unwrap();
        assert!(actual == expected, "{} is stale: run with UPDATE_BINDINGS=1 and review the diff", file.display());
    }
}
