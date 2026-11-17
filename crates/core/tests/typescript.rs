//! Every type the core exports to TypeScript has a declaration (`ts-rs` builds them at run time).

mod positive {
    use fresher_core::{
        Access, AccessStatus, AppRef, Availability, ByteSize, Capability, CapabilityState, CategoryDescriptor,
        CategoryId, CategoryInfo, CategoryShare, CategoryTotal, CommandError, Completeness, DangerLevel, Descent,
        Dictionary, EngineStatus, Event, Finding, FindingId, FindingKey, FindingOrder, FindingsPage, FindingsQuery,
        GuardedFolder, LastScan, LeftOutCount, Locale, MapPart, MapPhase, MeaningGroup, ModuleId, ModuleInfo,
        ModuleOutcome, MountKind, Need, NeedKind, NewFinding, Operation, Overview, PartKind, PlaceName, PlaceRow,
        PlaceShare, PlaceView, Refusal, RemainderReason, Removal, ScanId, ScanOutcome, SettingDescriptor, SettingKey,
        SettingKind, SettingProblem, SettingUpdate, SettingValue, SettingsFile, SettingsView, Sizes, StorageMap,
        Subject, Text, TextArg, TextKey, Timestamp, ToolCommand, ToolId, Unit, VolumeKind, VolumeMap,
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
            declaration::<Availability>(),
            declaration::<Capability>(),
            declaration::<CapabilityState>(),
            declaration::<GuardedFolder>(),
            declaration::<Need>(),
            declaration::<NeedKind>(),
            declaration::<VolumeKind>(),
            declaration::<ByteSize>(),
            declaration::<CategoryDescriptor>(),
            declaration::<CategoryId>(),
            declaration::<CategoryInfo>(),
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
            declaration::<LeftOutCount>(),
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

    #[test]
    fn the_typescript_declarations_are_written() {
        let declarations = [
            declaration::<StorageMap>(),
            declaration::<MapPhase>(),
            declaration::<VolumeMap>(),
            declaration::<MapPart>(),
            declaration::<PartKind>(),
            declaration::<RemainderReason>(),
            declaration::<MountKind>(),
            declaration::<MeaningGroup>(),
            declaration::<AppRef>(),
            declaration::<CategoryShare>(),
            declaration::<PlaceShare>(),
            declaration::<PlaceName>(),
            declaration::<PlaceView>(),
            declaration::<PlaceRow>(),
            declaration::<Descent>(),
        ];

        for declaration in &declarations {
            assert!(declaration.starts_with("type "), "{declaration}");
            assert!(!declaration.contains("bigint"), "a u64 must be a number: {declaration}");
        }
        assert!(declarations[2].contains("diskChanged"), "{}", declarations[2]);
        assert!(declarations[4].contains("outsideScan"), "{}", declarations[4]);
        assert!(declarations[8].contains("id: string"), "{}", declarations[8]);
    }

    #[test]
    fn a_finding_may_name_its_app() {
        assert!(declaration::<NewFinding>().contains("app?: string"), "{}", declaration::<NewFinding>());
    }
}
