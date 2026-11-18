//! The journal's records as the file holds them: a header line, then one JSON object per line tagged by `type`.
//!
//! These forms are the file's own, separate from the core's types and their IPC forms, so a change of an IPC type
//! never changes the file. Each converts from the core's record and back; a line whose values the core refuses (a text
//! key that is not one) does not convert and is skipped like a line that does not parse.

use std::collections::BTreeMap;

use fresher_core::{
    ActionKind, BeginRecord, ByteSize, CleanupId, Completeness, DangerLevel, EndRecord, Ended, EntryKind,
    FormatVersion, ItemIdentity, ItemRecord, JournalError, JournalItem, Method, PutBackOutcome, PutBackRecord,
    RecordedOutcome, Sizes, SkipCode, Text, TextArg, TextKey, Timestamp, TrashedItem, VolumeSpace,
};
use serde::{Deserialize, Serialize};

use super::path_text::StoredPath;

/// The `format` of the header line.
pub(super) const FORMAT: &str = "cleanup-journal";
/// The version of the records this build writes.
pub(super) const VERSION: FormatVersion = FormatVersion::new(1);

/// The first line of every journal file.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct Header {
    pub(super) format: String,
    pub(super) version: FormatVersion,
}

/// The header line of a file written at `version`, with its newline.
pub(super) fn header_line(version: FormatVersion) -> Result<Vec<u8>, JournalError> {
    to_line(&Header { format: FORMAT.to_owned(), version })
}

/// `value` as one line of the file: compact JSON (a newline inside a string is escaped) and a newline.
pub(super) fn to_line(value: &impl Serialize) -> Result<Vec<u8>, JournalError> {
    let mut line = serde_json::to_vec(value).map_err(|error| JournalError::Failed { reason: error.to_string() })?;
    line.push(b'\n');
    Ok(line)
}

/// One record after the header.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub(super) enum Line {
    Begin(BeginLine),
    Item(ItemLine),
    End(EndLine),
    PutBack(PutBackLine),
}

impl Line {
    /// The cleanup the record belongs to.
    pub(super) const fn cleanup(&self) -> u32 {
        match self {
            Self::Begin(line) => line.cleanup,
            Self::Item(line) => line.cleanup,
            Self::End(line) => line.cleanup,
            Self::PutBack(line) => line.cleanup,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct BeginLine {
    pub(super) cleanup: u32,
    at: u64,
    method: MethodForm,
    roots: Vec<StoredPath>,
    space: Vec<SpaceForm>,
    items: Vec<ItemForm>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct ItemForm {
    index: u32,
    path: Option<StoredPath>,
    identity: Option<IdentityForm>,
    action: ActionForm,
    category: String,
    key: String,
    reason: TextForm,
    danger: DangerForm,
    sizes: SizesForm,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct IdentityForm {
    device: u64,
    file: u64,
    kind: EntryKindForm,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct SizesForm {
    logical: u64,
    allocated: u64,
    reclaimable: u64,
    completeness: CompletenessForm,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct SpaceForm {
    root: StoredPath,
    available: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct TextForm {
    key: String,
    args: BTreeMap<String, ArgForm>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "kebab-case")]
enum ArgForm {
    Text(String),
    Count(u64),
    Size(u64),
    Date(u64),
    Path(StoredPath),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct ItemLine {
    pub(super) cleanup: u32,
    index: u32,
    outcome: OutcomeForm,
    trashed: Vec<TrashedForm>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
enum OutcomeForm {
    Moved,
    Emptied { moved: u32, skipped: u32 },
    Deleted,
    ToolDone,
    Skipped { reason: SkipCodeForm },
    Failed { status: Option<i32> },
    NotReached,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct TrashedForm {
    original: StoredPath,
    landed: StoredPath,
    identity: IdentityForm,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct EndLine {
    pub(super) cleanup: u32,
    at: u64,
    ended: EndedForm,
    space: Vec<SpaceForm>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct PutBackLine {
    pub(super) cleanup: u32,
    index: u32,
    at: u64,
    outcome: PutBackForm,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
enum PutBackForm {
    PutBack,
    NotTrashed,
    Gone,
    Replaced,
    Occupied,
    NoParent,
    Refused { reason: SkipCodeForm },
    Failed,
}

/// A field-less enum of the core mirrored by one of the file, with both conversions; the matches are exhaustive, so a
/// new variant in the core does not compile until the file has its form.
macro_rules! mirror {
    ($form:ident, $core:ident { $($variant:ident),+ $(,)? }) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
        #[serde(rename_all = "kebab-case")]
        enum $form {
            $($variant),+
        }

        impl From<$core> for $form {
            fn from(value: $core) -> Self {
                match value {
                    $($core::$variant => Self::$variant),+
                }
            }
        }

        impl From<$form> for $core {
            fn from(value: $form) -> Self {
                match value {
                    $($form::$variant => Self::$variant),+
                }
            }
        }
    };
}

mirror!(MethodForm, Method { Trash, Permanent });
mirror!(ActionForm, ActionKind { Remove, Empty, Tool });
mirror!(DangerForm, DangerLevel { Safe, Low, Medium, High });
mirror!(CompletenessForm, Completeness { Complete, Partial, Unknown });
mirror!(EntryKindForm, EntryKind { File, Folder, Symlink, Other });
mirror!(EndedForm, Ended { Completed, Cancelled, Stopped, Interrupted });
mirror!(
    SkipCodeForm,
    SkipCode {
        NotInSandbox,
        OutsideRoots,
        Protected,
        WholeFolder,
        Ignored,
        ThroughLink,
        Vanished,
        OtherVolume,
        Replaced,
        KindChanged,
        NotRemovable,
        MissingCapability,
        InUse,
        Grown,
        NoTrash,
        Covered,
        NotTheSubject,
        ForcedTool,
        CheckFailed,
    }
);

/// A record whose values the core refuses.
#[derive(Debug, thiserror::Error)]
#[error("a journal record holds a value the core refuses")]
pub(super) struct Refused;

impl BeginLine {
    pub(super) fn new(cleanup: CleanupId, record: &BeginRecord) -> Self {
        Self {
            cleanup: cleanup.get(),
            at: record.at.secs(),
            method: record.method.into(),
            roots: record.roots.iter().map(|root| StoredPath::from(root.as_path())).collect(),
            space: record.space.iter().map(SpaceForm::new).collect(),
            items: record.items.iter().map(ItemForm::new).collect(),
        }
    }

    pub(super) fn into_record(self) -> Result<BeginRecord, Refused> {
        Ok(BeginRecord {
            at: Timestamp::from_secs(self.at),
            method: self.method.into(),
            roots: self.roots.into_iter().map(|root| root.0).collect(),
            space: self.space.into_iter().map(SpaceForm::into_space).collect(),
            items: self.items.into_iter().map(ItemForm::into_item).collect::<Result<_, _>>()?,
        })
    }
}

impl ItemForm {
    fn new(item: &JournalItem) -> Self {
        Self {
            index: item.index,
            path: item.path.as_deref().map(StoredPath::from),
            identity: item.identity.map(IdentityForm::new),
            action: item.action.into(),
            category: item.category.clone(),
            key: item.key.clone(),
            reason: TextForm::new(&item.reason),
            danger: item.danger.into(),
            sizes: SizesForm::new(item.sizes),
        }
    }

    fn into_item(self) -> Result<JournalItem, Refused> {
        Ok(JournalItem {
            index: self.index,
            path: self.path.map(|path| path.0),
            identity: self.identity.map(IdentityForm::into_identity),
            action: self.action.into(),
            category: self.category,
            key: self.key,
            reason: self.reason.into_text()?,
            danger: self.danger.into(),
            sizes: self.sizes.into_sizes(),
        })
    }
}

impl IdentityForm {
    fn new(identity: ItemIdentity) -> Self {
        Self { device: identity.device(), file: identity.file(), kind: identity.kind().into() }
    }

    fn into_identity(self) -> ItemIdentity {
        ItemIdentity::new(self.device, self.file, self.kind.into())
    }
}

impl SizesForm {
    fn new(sizes: Sizes) -> Self {
        Self {
            logical: sizes.logical.bytes(),
            allocated: sizes.allocated.bytes(),
            reclaimable: sizes.reclaimable.bytes(),
            completeness: sizes.completeness.into(),
        }
    }

    fn into_sizes(self) -> Sizes {
        Sizes {
            logical: ByteSize::new(self.logical),
            allocated: ByteSize::new(self.allocated),
            reclaimable: ByteSize::new(self.reclaimable),
            completeness: self.completeness.into(),
        }
    }
}

impl SpaceForm {
    fn new(space: &VolumeSpace) -> Self {
        Self { root: StoredPath::from(space.root.as_path()), available: space.available.bytes() }
    }

    fn into_space(self) -> VolumeSpace {
        VolumeSpace { root: self.root.0, available: ByteSize::new(self.available) }
    }
}

impl TextForm {
    fn new(text: &Text) -> Self {
        let args = text.args().iter().map(|(name, arg)| (name.clone(), ArgForm::new(arg))).collect();
        Self { key: text.key().as_str().to_owned(), args }
    }

    fn into_text(self) -> Result<Text, Refused> {
        let key = TextKey::new(self.key).map_err(|_| Refused)?;
        Ok(self.args.into_iter().fold(Text::new(key), |text, (name, arg)| text.with(name, arg.into_arg())))
    }
}

impl ArgForm {
    fn new(arg: &TextArg) -> Self {
        match arg {
            TextArg::Text(text) => Self::Text(text.clone()),
            TextArg::Count(count) => Self::Count(*count),
            TextArg::Size(size) => Self::Size(size.bytes()),
            TextArg::Date(date) => Self::Date(date.secs()),
            TextArg::Path(path) => Self::Path(StoredPath::from(path.as_path())),
        }
    }

    fn into_arg(self) -> TextArg {
        match self {
            Self::Text(text) => TextArg::Text(text),
            Self::Count(count) => TextArg::Count(count),
            Self::Size(size) => TextArg::Size(ByteSize::new(size)),
            Self::Date(date) => TextArg::Date(Timestamp::from_secs(date)),
            Self::Path(path) => TextArg::Path(path.0),
        }
    }
}

impl ItemLine {
    pub(super) fn new(record: &ItemRecord) -> Self {
        Self {
            cleanup: record.cleanup.get(),
            index: record.index,
            outcome: OutcomeForm::new(record.outcome),
            trashed: record.trashed.iter().map(TrashedForm::new).collect(),
        }
    }

    pub(super) fn into_record(self) -> ItemRecord {
        ItemRecord {
            cleanup: CleanupId::new(self.cleanup),
            index: self.index,
            outcome: self.outcome.into_outcome(),
            trashed: self.trashed.into_iter().map(TrashedForm::into_trashed).collect(),
        }
    }
}

impl OutcomeForm {
    fn new(outcome: RecordedOutcome) -> Self {
        match outcome {
            RecordedOutcome::Moved => Self::Moved,
            RecordedOutcome::Emptied { moved, skipped } => Self::Emptied { moved, skipped },
            RecordedOutcome::Deleted => Self::Deleted,
            RecordedOutcome::ToolDone => Self::ToolDone,
            RecordedOutcome::Skipped { reason } => Self::Skipped { reason: reason.into() },
            RecordedOutcome::Failed { status } => Self::Failed { status },
            RecordedOutcome::NotReached => Self::NotReached,
            RecordedOutcome::Unknown => Self::Unknown,
        }
    }

    fn into_outcome(self) -> RecordedOutcome {
        match self {
            Self::Moved => RecordedOutcome::Moved,
            Self::Emptied { moved, skipped } => RecordedOutcome::Emptied { moved, skipped },
            Self::Deleted => RecordedOutcome::Deleted,
            Self::ToolDone => RecordedOutcome::ToolDone,
            Self::Skipped { reason } => RecordedOutcome::Skipped { reason: reason.into() },
            Self::Failed { status } => RecordedOutcome::Failed { status },
            Self::NotReached => RecordedOutcome::NotReached,
            Self::Unknown => RecordedOutcome::Unknown,
        }
    }
}

impl TrashedForm {
    fn new(trashed: &TrashedItem) -> Self {
        Self {
            original: StoredPath::from(trashed.original.as_path()),
            landed: StoredPath::from(trashed.landed.as_path()),
            identity: IdentityForm::new(trashed.identity),
        }
    }

    fn into_trashed(self) -> TrashedItem {
        TrashedItem { original: self.original.0, landed: self.landed.0, identity: self.identity.into_identity() }
    }
}

impl EndLine {
    pub(super) fn new(record: &EndRecord) -> Self {
        Self {
            cleanup: record.cleanup.get(),
            at: record.at.secs(),
            ended: record.ended.into(),
            space: record.space.iter().map(SpaceForm::new).collect(),
        }
    }

    pub(super) fn into_record(self) -> EndRecord {
        EndRecord {
            cleanup: CleanupId::new(self.cleanup),
            at: Timestamp::from_secs(self.at),
            ended: self.ended.into(),
            space: self.space.into_iter().map(SpaceForm::into_space).collect(),
        }
    }
}

impl PutBackLine {
    pub(super) fn new(record: &PutBackRecord) -> Self {
        Self {
            cleanup: record.cleanup.get(),
            index: record.index,
            at: record.at.secs(),
            outcome: PutBackForm::new(record.outcome),
        }
    }

    pub(super) fn into_record(self) -> PutBackRecord {
        PutBackRecord {
            cleanup: CleanupId::new(self.cleanup),
            index: self.index,
            at: Timestamp::from_secs(self.at),
            outcome: self.outcome.into_outcome(),
        }
    }
}

impl PutBackForm {
    fn new(outcome: PutBackOutcome) -> Self {
        match outcome {
            PutBackOutcome::PutBack => Self::PutBack,
            PutBackOutcome::NotTrashed => Self::NotTrashed,
            PutBackOutcome::Gone => Self::Gone,
            PutBackOutcome::Replaced => Self::Replaced,
            PutBackOutcome::Occupied => Self::Occupied,
            PutBackOutcome::NoParent => Self::NoParent,
            PutBackOutcome::Refused { reason } => Self::Refused { reason: reason.into() },
            PutBackOutcome::Failed => Self::Failed,
        }
    }

    fn into_outcome(self) -> PutBackOutcome {
        match self {
            Self::PutBack => PutBackOutcome::PutBack,
            Self::NotTrashed => PutBackOutcome::NotTrashed,
            Self::Gone => PutBackOutcome::Gone,
            Self::Replaced => PutBackOutcome::Replaced,
            Self::Occupied => PutBackOutcome::Occupied,
            Self::NoParent => PutBackOutcome::NoParent,
            Self::Refused { reason } => PutBackOutcome::Refused { reason: reason.into() },
            Self::Failed => PutBackOutcome::Failed,
        }
    }
}

/// Records for the tests of the journal: a cleanup of `count` items with every kind of value a record can hold.
#[cfg(test)]
pub(super) mod sample {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;
    use std::path::PathBuf;

    use fresher_core::{
        ActionKind, BeginRecord, ByteSize, CleanupId, DangerLevel, EndRecord, Ended, EntryKind, ItemIdentity,
        ItemRecord, JournalItem, Method, PutBackOutcome, PutBackRecord, RecordedOutcome, Sizes, SkipCode, Text,
        TextArg, TextKey, Timestamp, TrashedItem, VolumeSpace,
    };

    pub(in crate::journal) fn item(index: u32) -> JournalItem {
        let reason = Text::new(TextKey::new("demo.finding.reason").unwrap())
            .with("count", TextArg::Count(3))
            .with("size", TextArg::Size(ByteSize::new(4096)))
            .with("date", TextArg::Date(Timestamp::from_secs(1_700_000_000)))
            .with("app", TextArg::Text("Editor".into()))
            .with("where", TextArg::Path(PathBuf::from("/home/Library")));
        JournalItem {
            index,
            path: Some(PathBuf::from(format!("/home/Library/Caches/item-{index}"))),
            identity: Some(ItemIdentity::new(1, u64::from(index) + 100, EntryKind::Folder)),
            action: ActionKind::Remove,
            category: "demo.caches".into(),
            key: format!("item-{index}"),
            reason,
            danger: DangerLevel::Low,
            sizes: Sizes::complete(ByteSize::new(4000), ByteSize::new(4096)),
        }
    }

    pub(in crate::journal) fn begin(count: u32) -> BeginRecord {
        let mut items: Vec<JournalItem> = (0..count).map(item).collect();
        if let Some(last) = items.last_mut().filter(|_| count > 1) {
            *last = JournalItem { path: None, identity: None, action: ActionKind::Tool, ..last.clone() };
        }
        BeginRecord {
            at: Timestamp::from_secs(1_763_290_000),
            method: Method::Trash,
            roots: vec![PathBuf::from("/home"), PathBuf::from(OsStr::from_bytes(b"/Volumes/n\xe9e"))],
            space: vec![VolumeSpace { root: PathBuf::from("/home"), available: ByteSize::new(1 << 30) }],
            items,
        }
    }

    pub(in crate::journal) fn moved(cleanup: CleanupId, index: u32) -> ItemRecord {
        let trashed = TrashedItem {
            original: PathBuf::from(format!("/home/Library/Caches/item-{index}")),
            landed: PathBuf::from(format!("/home/.Trash/item-{index}")),
            identity: ItemIdentity::new(1, u64::from(index) + 100, EntryKind::Folder),
        };
        ItemRecord { cleanup, index, outcome: RecordedOutcome::Moved, trashed: vec![trashed] }
    }

    pub(in crate::journal) fn skipped(cleanup: CleanupId, index: u32) -> ItemRecord {
        ItemRecord { cleanup, index, outcome: RecordedOutcome::Skipped { reason: SkipCode::InUse }, trashed: vec![] }
    }

    pub(in crate::journal) fn end(cleanup: CleanupId) -> EndRecord {
        EndRecord {
            cleanup,
            at: Timestamp::from_secs(1_763_290_004),
            ended: Ended::Completed,
            space: vec![VolumeSpace { root: PathBuf::from("/home"), available: ByteSize::new(1 << 30) }],
        }
    }

    pub(in crate::journal) fn put_back(cleanup: CleanupId, index: u32, outcome: PutBackOutcome) -> PutBackRecord {
        PutBackRecord { cleanup, index, at: Timestamp::from_secs(1_763_300_000), outcome }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use serde_json::Value;

    /// Every object key of `value`, except the placeholder names inside a text's `args`.
    fn keys(value: &Value, into: &mut BTreeSet<String>) {
        match value {
            Value::Object(object) => {
                for (key, inner) in object {
                    into.insert(key.clone());
                    if key == "args" {
                        object_values(inner).for_each(|arg| keys(arg, into));
                    } else {
                        keys(inner, into);
                    }
                }
            }
            Value::Array(values) => values.iter().for_each(|inner| keys(inner, into)),
            _ => {}
        }
    }

    fn object_values(value: &Value) -> impl Iterator<Item = &Value> {
        value.as_object().into_iter().flat_map(serde_json::Map::values)
    }

    mod negative {
        use std::collections::BTreeSet;

        use fresher_core::{CleanupId, PutBackOutcome, RecordedOutcome, SkipCode};

        use super::keys;
        use crate::journal::records::{
            BeginLine, EndLine, ItemLine, Line, PutBackLine, VERSION, header_line, sample, to_line,
        };

        /// The fields the design names for the records (plan 008, section 8), and nothing more.
        const NAMED: &[&str] = &[
            "type",
            "format",
            "version",
            "cleanup",
            "at",
            "method",
            "roots",
            "space",
            "root",
            "available",
            "items",
            "index",
            "path",
            "identity",
            "device",
            "file",
            "kind",
            "action",
            "category",
            "key",
            "reason",
            "args",
            "value",
            "danger",
            "sizes",
            "logical",
            "allocated",
            "reclaimable",
            "completeness",
            "outcome",
            "moved",
            "skipped",
            "status",
            "trashed",
            "original",
            "landed",
            "ended",
            "bytes",
        ];

        #[test]
        fn a_record_never_holds_a_field_the_design_does_not_name() {
            let id = CleanupId::new(7);
            let mut emptied = sample::moved(id, 1);
            emptied.outcome = RecordedOutcome::Emptied { moved: 2, skipped: 1 };
            let mut failed = sample::skipped(id, 2);
            failed.outcome = RecordedOutcome::Failed { status: Some(1) };
            let lines = [
                header_line(VERSION).unwrap(),
                to_line(&Line::Begin(BeginLine::new(id, &sample::begin(3)))).unwrap(),
                to_line(&Line::Item(ItemLine::new(&sample::moved(id, 0)))).unwrap(),
                to_line(&Line::Item(ItemLine::new(&emptied))).unwrap(),
                to_line(&Line::Item(ItemLine::new(&failed))).unwrap(),
                to_line(&Line::Item(ItemLine::new(&sample::skipped(id, 2)))).unwrap(),
                to_line(&Line::End(EndLine::new(&sample::end(id)))).unwrap(),
                to_line(&Line::PutBack(PutBackLine::new(&sample::put_back(
                    id,
                    0,
                    PutBackOutcome::Refused { reason: SkipCode::Protected },
                ))))
                .unwrap(),
            ];
            let mut found = BTreeSet::new();
            for line in &lines {
                keys(&serde_json::from_slice(line).unwrap(), &mut found);
            }

            let named: BTreeSet<String> = NAMED.iter().map(|&name| name.to_owned()).collect();
            assert_eq!(found.difference(&named).collect::<Vec<_>>(), Vec::<&String>::new());
        }

        #[test]
        fn a_reason_with_a_key_the_core_refuses_does_not_convert() {
            let line = to_line(&Line::Begin(BeginLine::new(CleanupId::new(1), &sample::begin(1)))).unwrap();
            let text = String::from_utf8(line).unwrap().replace("demo.finding.reason", "Not A Key");

            let Line::Begin(begin) = serde_json::from_str(&text).unwrap() else { unreachable!("not a begin") };

            assert!(begin.into_record().is_err());
        }

        #[test]
        fn a_line_is_one_line_whatever_its_strings_hold() {
            let mut record = sample::begin(1);
            record.items[0].category = "two\nlines".into();

            let line = to_line(&Line::Begin(BeginLine::new(CleanupId::new(1), &record))).unwrap();

            assert_eq!(line.split(|&byte| byte == b'\n').count(), 2, "one newline, at the end");
            assert_eq!(line.last(), Some(&b'\n'));
        }
    }

    mod positive {
        use fresher_core::{CleanupId, PutBackOutcome, RecordedOutcome};

        use crate::journal::records::{
            BeginLine, EndLine, ItemLine, Line, PutBackLine, VERSION, header_line, sample, to_line,
        };

        fn read(line: &[u8]) -> Line {
            serde_json::from_slice(line).unwrap()
        }

        #[test]
        fn the_header_names_the_format_and_its_version() {
            assert_eq!(header_line(VERSION).unwrap(), b"{\"format\":\"cleanup-journal\",\"version\":1}\n");
        }

        #[test]
        fn every_record_reads_back_as_it_was_written() {
            let id = CleanupId::new(7);
            let begin = sample::begin(3);
            let mut emptied = sample::moved(id, 1);
            emptied.outcome = RecordedOutcome::Emptied { moved: 2, skipped: 1 };
            let put_back = sample::put_back(id, 0, PutBackOutcome::PutBack);

            let Line::Begin(read_begin) = read(&to_line(&Line::Begin(BeginLine::new(id, &begin))).unwrap()) else {
                unreachable!("not a begin")
            };
            assert_eq!(read_begin.cleanup, 7);
            assert_eq!(read_begin.into_record().unwrap(), begin);
            for item in [sample::moved(id, 0), emptied, sample::skipped(id, 2)] {
                let Line::Item(line) = read(&to_line(&Line::Item(ItemLine::new(&item))).unwrap()) else {
                    unreachable!("not an item")
                };
                assert_eq!(line.into_record(), item);
            }
            let Line::End(end) = read(&to_line(&Line::End(EndLine::new(&sample::end(id)))).unwrap()) else {
                unreachable!("not an end")
            };
            assert_eq!(end.into_record(), sample::end(id));
            let Line::PutBack(line) = read(&to_line(&Line::PutBack(PutBackLine::new(&put_back))).unwrap()) else {
                unreachable!("not a put back")
            };
            assert_eq!(line.into_record(), put_back);
        }

        #[test]
        fn a_record_is_tagged_with_its_type() {
            let line = to_line(&Line::PutBack(PutBackLine::new(&sample::put_back(
                CleanupId::new(7),
                0,
                PutBackOutcome::PutBack,
            ))))
            .unwrap();

            assert_eq!(
                String::from_utf8(line).unwrap(),
                "{\"type\":\"put-back\",\"cleanup\":7,\"index\":0,\"at\":1763300000,\"outcome\":{\"kind\":\"put-back\"}}\n"
            );
        }
    }
}
