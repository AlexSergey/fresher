//! Every type the core exports to TypeScript has a declaration (`ts-rs` builds them at run time).

mod positive {
    use fresher_core::{
        Access, AccessStatus, ByteSize, CategoryDescriptor, CategoryId, CategoryTotal, CommandError, Completeness,
        DangerLevel, Dictionary, EngineStatus, Event, Finding, FindingId, FindingKey, FindingOrder, FindingsPage,
        FindingsQuery, LastScan, Locale, ModuleId, ModuleInfo, ModuleOutcome, NewFinding, Operation, Overview, Refusal,
        Removal, ScanId, ScanOutcome, SettingDescriptor, SettingKey, SettingKind, SettingProblem, SettingUpdate,
        SettingValue, SettingsFile, SettingsView, Sizes, Subject, Text, TextArg, TextKey, Timestamp, ToolCommand,
        ToolId, Unit,
    };
    use ts_rs::{Config, TS};

    fn declaration<T: TS>() -> String {
        T::decl(&Config::new())
    }

    #[test]
    fn every_exported_type_has_a_declaration() {
        let declarations = [
            declaration::<Access>(),
            declaration::<AccessStatus>(),
            declaration::<ByteSize>(),
            declaration::<CategoryDescriptor>(),
            declaration::<CategoryId>(),
            declaration::<CategoryTotal>(),
            declaration::<CommandError>(),
            declaration::<Completeness>(),
            declaration::<DangerLevel>(),
            declaration::<Dictionary>(),
            declaration::<EngineStatus>(),
            declaration::<Event>(),
            declaration::<Finding>(),
            declaration::<FindingId>(),
            declaration::<FindingKey>(),
            declaration::<FindingOrder>(),
            declaration::<FindingsPage>(),
            declaration::<FindingsQuery>(),
            declaration::<LastScan>(),
            declaration::<Locale>(),
            declaration::<ModuleId>(),
            declaration::<ModuleInfo>(),
            declaration::<ModuleOutcome>(),
            declaration::<NewFinding>(),
            declaration::<Operation>(),
            declaration::<Overview>(),
            declaration::<Refusal>(),
            declaration::<Removal>(),
            declaration::<ScanId>(),
            declaration::<ScanOutcome>(),
            declaration::<SettingDescriptor>(),
            declaration::<SettingKey>(),
            declaration::<SettingKind>(),
            declaration::<SettingProblem>(),
            declaration::<SettingUpdate>(),
            declaration::<SettingValue>(),
            declaration::<SettingsFile>(),
            declaration::<SettingsView>(),
            declaration::<Sizes>(),
            declaration::<Subject>(),
            declaration::<Text>(),
            declaration::<TextArg>(),
            declaration::<TextKey>(),
            declaration::<Timestamp>(),
            declaration::<ToolCommand>(),
            declaration::<ToolId>(),
            declaration::<Unit>(),
        ];

        for declaration in &declarations {
            assert!(declaration.starts_with("type "), "{declaration}");
            assert!(!declaration.contains("bigint"), "a u64 must be a number: {declaration}");
        }
    }
}
