//! What becomes of each first-level entry of Downloads: the tables of the design's section 3.4, row by row, on plain
//! values. Nothing here reads the disk or asks the system.

use fresher_core::{BundleId, ByteSize, InstalledApp, Receipt, RunningApp, SystemVersion, Timestamp};

use crate::gathered::Entry;
use crate::name::{Named, Version};
use crate::unfinished::{Form, Row, ends_with_longer};

const SECONDS_PER_DAY: u64 = 24 * 60 * 60;

/// A big file is at least this many bytes of logical size, as Finder counts a gigabyte.
const LARGE: u64 = 1_000_000_000;

/// What a resumable download holds.
const INFO_PLIST: &str = "Info.plist";
/// What a macOS installer holds.
const CONTENTS: &str = "Contents";

/// What the scan knows, read once in `finish`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Scan {
    pub(crate) now: Timestamp,
    /// The setting `downloads.older-than-days`.
    pub(crate) days: u32,
    pub(crate) restored: bool,
    /// The walk saw Downloads in full: what is not gathered is not there.
    pub(crate) place_complete: bool,
}

/// Why a recognized entry is kept (design, section 3.5), in the order the module declares them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Reason {
    NoSystem,
    AppsUnread,
    BrowserOpen,
    Recent,
    NotOlder,
    NewerMacos,
    ScanAgain,
    Special,
    NotYours,
}

impl Reason {
    pub(crate) const ALL: [Self; 9] = [
        Self::NoSystem,
        Self::AppsUnread,
        Self::BrowserOpen,
        Self::Recent,
        Self::NotOlder,
        Self::NewerMacos,
        Self::ScanAgain,
        Self::Special,
        Self::NotYours,
    ];

    /// The reason's text key.
    pub(crate) const fn key(self) -> &'static str {
        match self {
            Self::NoSystem => "downloads.left-out.no-system",
            Self::AppsUnread => "downloads.left-out.apps-unread",
            Self::BrowserOpen => "downloads.left-out.browser-open",
            Self::Recent => "downloads.left-out.recent",
            Self::NotOlder => "downloads.left-out.not-older",
            Self::NewerMacos => "downloads.left-out.newer-macos",
            Self::ScanAgain => "downloads.left-out.scan-again",
            Self::Special => "downloads.left-out.special",
            Self::NotYours => "downloads.left-out.not-yours",
        }
    }
}

/// Why an entry is offered, with what its reason's text says.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum OfferReason {
    Unfinished {
        days: u32,
    },
    SafariResumable {
        days: u32,
    },
    /// Proven by its name: the app it installed.
    Installer {
        app: BundleId,
        name: String,
        version: Option<String>,
    },
    /// Proven by a receipt: when it was installed.
    InstallerReceipt {
        installed: Timestamp,
    },
    MacosInstaller {
        running: SystemVersion,
    },
    LargeUntouched {
        size: ByteSize,
        since: Timestamp,
    },
}

/// What becomes of an entry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Verdict {
    /// Not recognized: the user's, and nothing is said.
    NotClaimed,
    /// Recognized and kept, reported as left out with the reason.
    LeftOut(Reason),
    Offered {
        reason: OfferReason,
    },
}

/// What proves an installer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Proof {
    /// A receipt of the package, written at or after the installer's newest change.
    Receipt { installed: Timestamp },
    /// No receipt: what the name rule found.
    Name(Named),
}

/// The installed apps as the system listed them.
#[derive(Clone, Copy, Debug)]
pub(crate) enum AppsState<'a> {
    /// They could not be listed.
    Unlisted,
    /// Some apps or folders could not be read.
    Partial,
    Complete(&'a [InstalledApp]),
}

/// The unfinished-download table: a file ending with a file row's suffix.
///
/// 1. a restored scan: `ScanAgain`; 2. the running apps cannot be listed: `NoSystem`; 3. an app of the row is
///    running: `BrowserOpen`; 4. its newest change unknown, in the future or within the days: `Recent`; 5. offered.
pub(crate) fn unfinished(file: &Entry, row: &Row, running: Result<&[RunningApp], ()>, scan: &Scan) -> Verdict {
    if file.kind != Form::File {
        return Verdict::NotClaimed;
    }
    if scan.restored {
        return Verdict::LeftOut(Reason::ScanAgain);
    }
    let Ok(running) = running else { return Verdict::LeftOut(Reason::NoSystem) };
    if is_open(row, running) {
        return Verdict::LeftOut(Reason::BrowserOpen);
    }
    if !is_old(file.newest_change(), scan) {
        return Verdict::LeftOut(Reason::Recent);
    }
    Verdict::Offered { reason: OfferReason::Unfinished { days: scan.days } }
}

/// Whether `sibling`, the file of the final name beside an offered `.part`, is offered with it: a regular file of 0
/// bytes, in a place seen in full.
pub(crate) fn placeholder(part_offered: bool, sibling: Option<&Entry>, scan: &Scan) -> bool {
    part_offered
        && !scan.restored
        && scan.place_complete
        && sibling.is_some_and(|sibling| sibling.kind == Form::File && sibling.sizes.logical == ByteSize::ZERO)
}

/// The resumable-download table, for a bundle ending with the bundle row's suffix: not claimed without `Info.plist`;
/// rows 1 to 4 of the unfinished table with the newest change inside it; then not read in full: `ScanAgain`; a
/// special entry inside: `Special`; an entry not the user's: `NotYours`; otherwise offered.
pub(crate) fn resumable(bundle: &Entry, row: &Row, running: Result<&[RunningApp], ()>, scan: &Scan) -> Verdict {
    if bundle.kind != Form::Bundle || !bundle.holds(INFO_PLIST) {
        return Verdict::NotClaimed;
    }
    if scan.restored {
        return Verdict::LeftOut(Reason::ScanAgain);
    }
    let Ok(running) = running else { return Verdict::LeftOut(Reason::NoSystem) };
    if is_open(row, running) {
        return Verdict::LeftOut(Reason::BrowserOpen);
    }
    if !is_old(bundle.inside.and_then(|inside| inside.newest_change), scan) {
        return Verdict::LeftOut(Reason::Recent);
    }
    match kept_inside(bundle) {
        Some(reason) => Verdict::LeftOut(reason),
        None => Verdict::Offered { reason: OfferReason::SafariResumable { days: scan.days } },
    }
}

/// The macOS-installer table, for a bundle named in `macos.toml` that installs `version`: not claimed without
/// `Contents`; 1. a restored scan or not read in full: `ScanAgain`; 2. the running macOS unknown: `NoSystem`; 3. it
/// is older than `version`: `NewerMacos`; 4. `Special` / `NotYours`; 5. offered.
pub(crate) fn macos_installer(
    bundle: &Entry,
    version: SystemVersion,
    running_os: Result<SystemVersion, ()>,
    scan: &Scan,
) -> Verdict {
    if bundle.kind != Form::Bundle || !bundle.holds(CONTENTS) {
        return Verdict::NotClaimed;
    }
    if scan.restored || !bundle.inside.is_some_and(|inside| inside.complete) {
        return Verdict::LeftOut(Reason::ScanAgain);
    }
    let Ok(running) = running_os else { return Verdict::LeftOut(Reason::NoSystem) };
    if running < version {
        return Verdict::LeftOut(Reason::NewerMacos);
    }
    match kept_inside(bundle) {
        Some(reason) => Verdict::LeftOut(reason),
        None => Verdict::Offered { reason: OfferReason::MacosInstaller { running } },
    }
}

/// A disk image or a package.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Package {
    DiskImage,
    Flat,
}

/// The package `name` is, and its stem (the name without its extension): a name ending `.dmg` or `.pkg`, case
/// ignored.
pub(crate) fn package_of(name: &str) -> Option<(&str, Package)> {
    [(".dmg", Package::DiskImage), (".pkg", Package::Flat)].into_iter().find_map(|(extension, package)| {
        let cut = name.len().checked_sub(extension.len())?;
        let (stem, end) = (name.get(..cut)?, name.get(cut..)?);
        end.eq_ignore_ascii_case(extension).then_some((stem, package))
    })
}

/// When a receipt proves `file`: one whose package file is the file's name (ASCII case ignored), installed at or
/// after the file's newest change; the latest such install.
pub(crate) fn receipt_proof(file: &Entry, receipts: &[Receipt]) -> Option<Timestamp> {
    let time = file.newest_change()?;
    receipts
        .iter()
        .filter(|receipt| receipt.package_file.eq_ignore_ascii_case(&file.name) && receipt.installed >= time)
        .map(|receipt| receipt.installed)
        .max()
}

/// The installer table, for a `.dmg` or `.pkg` file and what proves it:
///
/// 1. no receipt, and the installed apps cannot be listed: `NoSystem`;
/// 2. no receipt, and the name names no installed app or two: not claimed;
/// 3. no receipt, and the list of installed apps is partial: `AppsUnread`;
/// 4. a restored scan: `ScanAgain`;
/// 5. by name, the app's change unknown or before the installer's newest change, or the installer's version newer
///    than the app's or not comparable with it: `NotOlder`;
/// 6. offered.
pub(crate) fn installer(file: &Entry, proof: Proof, apps: AppsState<'_>, scan: &Scan) -> Verdict {
    if file.kind != Form::File {
        return Verdict::NotClaimed;
    }
    let named = match proof {
        Proof::Receipt { .. } if scan.restored => return Verdict::LeftOut(Reason::ScanAgain),
        Proof::Receipt { installed } => {
            return Verdict::Offered { reason: OfferReason::InstallerReceipt { installed } };
        }
        Proof::Name(named) => named,
    };
    if matches!(apps, AppsState::Unlisted) {
        return Verdict::LeftOut(Reason::NoSystem);
    }
    let Named::One { app, version } = named else { return Verdict::NotClaimed };
    let AppsState::Complete(apps) = apps else { return Verdict::LeftOut(Reason::AppsUnread) };
    if scan.restored {
        return Verdict::LeftOut(Reason::ScanAgain);
    }
    let Some(app) = apps.get(app) else { return Verdict::NotClaimed };
    let installed_after = app.changed.zip(file.newest_change()).is_some_and(|(changed, time)| changed >= time);
    let as_new = match version {
        None => true,
        Some(version) => app.version.as_deref().and_then(Version::of_app).is_some_and(|installed| version <= installed),
    };
    if !installed_after || !as_new {
        return Verdict::LeftOut(Reason::NotOlder);
    }
    let reason =
        OfferReason::Installer { app: app.bundle_id.clone(), name: app.name.clone(), version: app.version.clone() };
    Verdict::Offered { reason }
}

/// The large-untouched table, for a file no rule above claimed: not claimed when it is not a regular file, is a
/// `.dmg` or `.pkg`, or is under 1 GB; 1. a restored scan: `ScanAgain`; 2. its newest change unknown, in the future
/// or within the days: `Recent`; 3. offered.
pub(crate) fn large(file: &Entry, scan: &Scan) -> Verdict {
    if file.kind != Form::File || package_of(&file.name).is_some() || file.sizes.logical.bytes() < LARGE {
        return Verdict::NotClaimed;
    }
    if scan.restored {
        return Verdict::LeftOut(Reason::ScanAgain);
    }
    let newest = file.newest_change();
    match newest.filter(|_| is_old(newest, scan)) {
        Some(since) => Verdict::Offered { reason: OfferReason::LargeUntouched { size: file.sizes.logical, since } },
        None => Verdict::LeftOut(Reason::Recent),
    }
}

/// The name of the placeholder of `part`, when its row has one: the name without the row's suffix.
pub(crate) fn placeholder_name<'a>(part: &'a Entry, row: &Row) -> Option<&'a str> {
    if !row.placeholder || !ends_with_longer(&part.name, &row.suffix) {
        return None;
    }
    part.name.get(..part.name.len() - row.suffix.len())
}

/// An app of `row` is running.
fn is_open(row: &Row, running: &[RunningApp]) -> bool {
    running.iter().any(|app| app.bundle_ids.iter().any(|id| row.apps.contains(id)))
}

/// `time` is known and at least the days before now.
fn is_old(time: Option<Timestamp>, scan: &Scan) -> bool {
    let threshold = u64::from(scan.days).saturating_mul(SECONDS_PER_DAY);
    time.is_some_and(|time| time <= scan.now && scan.now.secs() - time.secs() >= threshold)
}

/// Why a bundle read in full is kept for what it holds: a special entry inside, or one not the user's.
fn kept_inside(bundle: &Entry) -> Option<Reason> {
    let Some(inside) = bundle.inside.filter(|inside| inside.complete) else { return Some(Reason::ScanAgain) };
    if inside.special {
        return Some(Reason::Special);
    }
    inside.foreign.then_some(Reason::NotYours)
}

#[cfg(test)]
mod tests;
