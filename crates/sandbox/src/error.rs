use std::path::PathBuf;

use fresher_core::{DangerLevel, IdError};
use fresher_testing::FixtureError;

use crate::MapPartName;

/// Why a case file was refused; every error about one case names it.
#[derive(Debug, thiserror::Error)]
pub enum CaseError {
    /// The file is not TOML of the case format (an unknown key at the top or in `[case.expect]` included).
    #[error("the case file is not valid: {source}")]
    Parse { source: Box<toml::de::Error> },
    /// The `module` is not a valid module id.
    #[error("the module `{module}` of the case file is not a module id: {source}")]
    Module { module: String, source: IdError },
    /// Two cases have one name.
    #[error("case \"{case}\": another case has the same name")]
    DuplicateName { case: String },
    /// A negative case follows a positive one.
    #[error("case \"{case}\": a negative case comes after a positive one; negatives go first")]
    PositiveBeforeNegative { case: String },
    /// A positive case has no `[case.expect]`.
    #[error("case \"{case}\": a positive case needs `[case.expect]`")]
    PositiveWithoutExpect { case: String },
    /// A negative case has a `[case.expect]`.
    #[error("case \"{case}\": a negative case expects nothing; remove its `[case.expect]`")]
    NegativeWithExpect { case: String },
    /// A `reason` without `after-cleanup = "skipped"`, or `skipped` without a `reason`.
    #[error("case \"{case}\": `reason` is given with `after-cleanup = \"skipped\"`, and only with it")]
    SkipReason { case: String },
    /// The case's files are not of the tree vocabulary (an unknown key included).
    #[error("case \"{case}\": the files are not valid: {source}")]
    Files { case: String, source: Box<toml::de::Error> },
    /// The case's files break a check of the tree: a path twice, an age, the fill cap.
    #[error("case \"{case}\": {source}")]
    Tree { case: String, source: FixtureError },
    /// A path, marker or hard link target that is not relative, plain and inside the sandbox.
    #[error("case \"{case}\": the path {path} must be relative, plain and inside the sandbox")]
    PathOutsideSandbox { case: String, path: PathBuf },
    /// An app is not in `Applications/`, `System/Applications/` or `home/Applications/`, or one folder below one.
    #[error(
        "case \"{case}\": the app {path} must be in Applications/, System/Applications/ or home/Applications/, \
         or one folder below one"
    )]
    AppOutsideAppsFolders { case: String, path: PathBuf },
    /// The case writes something other than an app into the fake home's apps folder.
    #[error("case \"{case}\": {path} is in home/Applications/, which holds apps only; declare it as a `[[case.app]]`")]
    NotAnAppInHomeApps { case: String, path: PathBuf },
    /// A `[[case.running]]` without a bundle id: nothing could tell it is open.
    #[error("case \"{case}\": the running app \"{name}\" needs at least one bundle id in `bundle-ids`")]
    RunningWithoutBundleId { case: String, name: String },
    /// A `[[case.running]]` bundle id the app would not read.
    #[error("case \"{case}\": `{id}` of a running app is not a bundle id")]
    RunningBundleId { case: String, id: String },
    /// A `[[case.elsewhere]]` whose name is empty or not one plain file name.
    #[error(
        "case \"{case}\": the app Spotlight finds elsewhere needs a `name` that is one plain file name, not {name:?}"
    )]
    ElsewhereName { case: String, name: String },
    /// A `[[case.elsewhere]]` bundle id the app would not read.
    #[error("case \"{case}\": `{id}` of an app Spotlight finds elsewhere is not a bundle id")]
    ElsewhereBundleId { case: String, id: String },
    /// An `access` key that is no guarded folder and not other apps' data.
    #[error(
        "case \"{case}\": `access` names `{key}`: only the guarded folders desktop, documents and downloads, and \
         other-apps-data"
    )]
    AccessKey { case: String, key: String },
    /// A `macos` that is no version (`15.4`).
    #[error("case \"{case}\": `macos` is {value:?}, not a macOS version such as \"15.4\"")]
    MacosVersion { case: String, value: String },
    /// A `[[case.receipt]]` whose package is not one plain file name ending in `.pkg`.
    #[error("case \"{case}\": a receipt's `package` must be one plain file name ending in .pkg, not {package:?}")]
    ReceiptPackage { case: String, package: String },
    /// A `[[case.receipt]]` whose `installed` is not an age.
    #[error("case \"{case}\": the receipt of {package} has `installed` {installed:?}, which is not an age")]
    ReceiptInstalled { case: String, package: String, installed: String },
    /// A `[[case.tool]]` program that is absolute or has a component that is not a plain name.
    #[error(
        "case \"{case}\": the tool's program {program:?} must be relative to the sandbox, every component a plain name"
    )]
    ToolProgram { case: String, program: String },
    /// A `[[case.tool]]` argument with a NUL.
    #[error("case \"{case}\": an argument of the tool {program} holds a NUL")]
    ToolArgument { case: String, program: String },
    /// A `[[case.tool]]` output over 64 KiB.
    #[error("case \"{case}\": the output of the tool {program} is over 64 KiB")]
    ToolOutput { case: String, program: String },
    /// A `[[case.insight]]` identity that is not a kebab-case name.
    #[error("case \"{case}\": the insight `{identity}` is not named in kebab case")]
    InsightIdentity { case: String, identity: String },
    /// A `[[case.insight]]` with more parts carrying a command than parts.
    #[error("case \"{case}\": the insight `{identity}` has {commands} commands for {parts} parts")]
    InsightCommands { case: String, identity: String, commands: u32, parts: u32 },
    /// Two `[[case.insight]]` of one file expect one identity.
    #[error("case \"{case}\": the insight `{identity}` is also expected by case \"{other}\"")]
    InsightTwice { case: String, other: String, identity: String },
    /// Two cases write one path.
    #[error("case \"{case}\": the path {path} is also written by case \"{other}\"")]
    PathWrittenTwice { case: String, other: String, path: PathBuf },
    /// A positive's `item` is not one of its case's paths or their folders.
    #[error("case \"{case}\": the item {item} is not one of the case's paths or their folders")]
    ItemNotInCase { case: String, item: PathBuf },
    /// The `category` is not a category id `<module>.<name>`.
    #[error("case \"{case}\": `{category}` is not a category id: {source}")]
    CategoryId { case: String, category: String, source: IdError },
    /// The `category` is not among the module's categories.
    #[error("case \"{case}\": the category `{category}` is not in the module's descriptor")]
    UnknownCategory { case: String, category: String },
    /// A `[[case.left-out]]` reason that is not a text key.
    #[error("case \"{case}\": `{reason}` is not a text key")]
    LeftOutReason { case: String, reason: String },
    /// A `[[case.left-out]]` with `items = 0`: it expects nothing.
    #[error("case \"{case}\": a left-out of `{category}` needs `items` of at least 1")]
    LeftOutWithoutItems { case: String, category: String },
    /// A `[[case.left-out]]` reason that is not one of the problems its category's module declares.
    #[error("case \"{case}\": `{reason}` is not a problem the module of `{category}` declares")]
    UndeclaredReason { case: String, category: String, reason: String },
    /// The expected danger is below the category's level.
    #[error("case \"{case}\": the danger {danger:?} is below the category's {floor:?}")]
    DangerBelowCategory { case: String, danger: DangerLevel, floor: DangerLevel },
    /// `[case.map]` names `explained` on a negative case, or `your-folders` on a positive one.
    #[error(
        "case \"{case}\": a negative case's files are in `your-folders`, a positive's are `explained`, not {part:?}"
    )]
    MapOnTheWrongKind { case: String, part: MapPartName },
    /// `[case.map]` has no `at` and the case's first file is at the sandbox's root, which has no folder to show.
    #[error(
        "case \"{case}\": the case's first file is at the sandbox's root; give `[case.map]` an `at`, a folder of the case"
    )]
    MapWithoutFolder { case: String },
    /// `[case.map]`'s `at` is not one of the case's paths or their folders.
    #[error("case \"{case}\": the map's `at` {at} is not one of the case's paths or their folders")]
    MapAtOutsideTheCase { case: String, at: PathBuf },
    /// A `[settings]` key that is not `<module>.<name>` in kebab case.
    #[error("the setting `{key}` is not a key `<module>.<name>` in kebab case")]
    SettingKey { key: String },
    /// `[case.largest]`'s `shown` is none of the four.
    #[error("case \"{case}\": `[case.largest]` shows {shown:?}; `shown` is offered, absent, withheld or not-seen")]
    LargestShown { case: String, shown: String },
    /// `[case.largest]` has a `withheld` without `shown = "withheld"`, or `shown = "withheld"` without one.
    #[error("case \"{case}\": `[case.largest]` gives `withheld` with `shown = \"withheld\"`, and only with it")]
    LargestWithheld { case: String },
    /// `[case.largest]`'s `withheld` is no reason a row is withheld for.
    #[error("case \"{case}\": `{reason}` in `[case.largest]` is not a reason a row is withheld for")]
    LargestWithheldReason { case: String, reason: String },
    /// `[case.largest]` has a `kind` or an `unpacked` without `shown = "offered"`.
    #[error("case \"{case}\": `[case.largest]` gives `{field}` with `shown = \"offered\"` only")]
    LargestOfferedOnly { case: String, field: &'static str },
    /// `[case.largest]`'s `kind` is no kind of a row.
    #[error("case \"{case}\": `{kind}` in `[case.largest]` is not a kind of a row")]
    LargestKind { case: String, kind: String },
    /// `[case.largest]`'s `unpacked` is not one plain file name.
    #[error("case \"{case}\": `unpacked` in `[case.largest]` must be one plain folder name, not {name:?}")]
    LargestUnpacked { case: String, name: String },
    /// `[case.largest]` has no `at` where one is required (withheld, not-seen) or the case writes no file to default to.
    #[error("case \"{case}\": `[case.largest]` needs an `at`, one of the case's paths or their folders")]
    LargestWithoutAt { case: String },
    /// `[case.largest]`'s `at` is not a plain path among the case's paths or their folders.
    #[error("case \"{case}\": the largest `at` {at} is not one of the case's paths or their folders")]
    LargestAtOutsideTheCase { case: String, at: PathBuf },
    /// A `core.own-file` expectation in a case without `[case.largest]`: nothing would pick the file.
    #[error("case \"{case}\": `core.own-file` is picked on the Largest screen; add a `[case.largest]`")]
    OwnFileWithoutLargest { case: String },
    /// A `[settings]` value that is not a bool, an integer or a string.
    #[error("the setting `{key}` must be a bool, an integer or a string")]
    SettingValue { key: String },
    /// Two case files set one setting to different values: a sandbox has one value.
    #[error("the setting `{key}` is set to different values by two case files; a sandbox has one")]
    SettingTwice { key: String },
    /// A demo world names a `module`: it shows every module at once.
    #[error("the case file `{feature}` is a demo world: it shows every module, so it names no `module`")]
    DemoWithModule { feature: String },
    /// A negative or positive case in a demo world, whose cases are scenery only.
    #[error("case \"{case}\": a demo world has scenery cases only; give it `kind = \"scenery\"`")]
    CheckedCaseInDemo { case: String },
    /// A scenery case in a case file without a `[demo]` header.
    #[error("case \"{case}\": a scenery case belongs to a demo world; its case file needs a `[demo]` header")]
    SceneryOutsideDemo { case: String },
    /// A scenery case with `[case.expect]`, `[case.map]`, `[[case.left-out]]` or `[[case.insight]]`: nothing it shows
    /// is checked.
    #[error("case \"{case}\": a scenery case is not checked; remove its `expect`, `map`, `left-out` and `insight`")]
    SceneryWithExpectation { case: String },
    /// The `[demo]` header breaks a check; the reason names the field.
    #[error("the `[demo]` header of `{feature}` is not valid: {reason}")]
    Demo { feature: String, reason: String },
    /// The cases together break a check of the tree, such as the fill cap of a sandbox.
    #[error("the cases do not make one sandbox: {source}")]
    Sandbox { source: FixtureError },
}

/// Why a sandbox could not be opened or generated; the generator removes nothing whatever the error.
#[derive(Debug, thiserror::Error)]
pub enum SandboxError {
    /// The folder holds no valid marker of the user.
    #[error("{folder} holds no valid sandbox marker")]
    NoMarker { folder: PathBuf },
    /// The folder is not empty and holds no valid marker: it is not a sandbox, so nothing is written into it.
    #[error("{folder} is not empty and holds no valid sandbox marker; choose an empty folder or a sandbox")]
    NotASandbox { folder: PathBuf },
    /// The real Mac's protection refuses the folder: the home itself, a protected place or an ancestor of one.
    #[error("{folder} is a protected place of this Mac; a sandbox goes below it, never into it")]
    Protected { folder: PathBuf },
    /// The path is a link or passes through one.
    #[error("{path} is a link or passes through one")]
    ThroughLink { path: PathBuf },
    /// The path is not a folder.
    #[error("{path} is not a folder")]
    NotAFolder { path: PathBuf },
    /// The part of the path that does not exist yet holds `.` or `..`.
    #[error("{path} has `.` or `..` below the folders that exist; give a plain path")]
    NotPlain { path: PathBuf },
    /// A file the generator would rewrite has a name it did not make, a hard link that may lie outside the sandbox.
    #[error("{path} has a hard link the generator did not make; it is not rewritten, so the sandbox is refused")]
    SharedFile { path: PathBuf },
    /// A part of the sandbox's layout is missing or not a folder.
    #[error("the sandbox folder {path} is missing")]
    MissingPart { path: PathBuf },
    /// Two case files write one path into one sandbox.
    #[error("the path {path} is written by both `{first}` and `{second}`")]
    PathWrittenTwice { path: PathBuf, first: String, second: String },
    /// Two demo worlds state different volumes: one sandbox is on one disk.
    #[error("the demo worlds state two volumes, {first} and {second}; a sandbox is on one")]
    VolumesDiffer { first: String, second: String },
    /// Two cases state different macOS versions: one sandbox has one.
    #[error("the cases state two macOS versions, {first} and {second}; a sandbox has one")]
    MacosVersions { first: String, second: String },
    /// Two cases state one call of a program (the same program and arguments) with different answers.
    #[error("the tool {program} is stated twice with the same arguments and different answers")]
    ToolTwice { program: PathBuf },
    /// A case file does not make one tree.
    #[error("{source}")]
    Case { source: CaseError },
    /// The cases' files could not be written.
    #[error("cannot write the cases: {source}")]
    Tree { source: FixtureError },
    /// The case files' settings could not be written into the data folder's settings file.
    #[error("cannot write the settings of the cases: {reason}")]
    Settings { reason: String },
    /// The sandbox's `world.toml` is not of the world's format or names an app the app would not read.
    #[error("{path} is not a valid world: {reason}")]
    World { path: PathBuf, reason: String },
    /// A file system call failed.
    #[error("{path}: {source}")]
    Io { path: PathBuf, source: std::io::Error },
}
