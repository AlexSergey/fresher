//! The worker loop: take a job, read the folder, match and fold its entries, send one report to the aggregator.

use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt as _;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc::SyncSender;

use fresher_core::{
    CACHEDIR_SIGNATURE, CACHEDIR_TAG, CancelToken, EntryKind, FolderIdentity, FolderListing, FolderReader, Inside,
    MarkerCheck, ReadError, Sizes, is_valid_cachedir_tag,
};

use crate::matcher::entry::sibling_mask;
use crate::matcher::state::TrieState;
use crate::matcher::{Hits, Matcher};
use crate::tree::fold::{FolderBatch, SharedFile, fold};
use crate::tree::node::{Held, Hit, NodeIndex, Sums};
use crate::tree::times::Times;
use crate::walk::boundary::{Boundary, Decision};
use crate::walk::errors;
use crate::walk::queue::{Job, WorkQueue};

/// What every worker shares, read-only.
#[derive(Clone, Copy)]
pub(crate) struct Context<'a> {
    pub(crate) matcher: &'a Matcher,
    pub(crate) boundary: &'a Boundary,
    pub(crate) reader: &'a dyn FolderReader,
    pub(crate) queue: &'a WorkQueue,
    pub(crate) cancel: &'a CancelToken,
    pub(crate) threshold: u64,
    /// The user the entries' access is read for.
    pub(crate) user: Option<u32>,
}

/// A subfolder of a listing, with what the walk decided about it; in listing order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Child {
    pub(crate) path: PathBuf,
    pub(crate) identity: FolderIdentity,
    pub(crate) state: TrieState,
    pub(crate) decision: Decision,
    /// An interest claims it whole: its subtree is summed, not kept.
    pub(crate) whole: bool,
    /// Its times in the listing, for its own hits.
    pub(crate) times: Times,
}

/// What one job gives the aggregator.
#[derive(Debug)]
pub(crate) struct Report {
    pub(crate) node: NodeIndex,
    pub(crate) opaque: Option<NodeIndex>,
    pub(crate) root_device: u64,
    pub(crate) body: Body,
}

#[derive(Debug)]
pub(crate) enum Body {
    /// The folder could not be read.
    Failed(ReadError),
    /// A folder outside any opaque subtree.
    Folder(FolderReport),
    /// A folder whose insides are summed: inside an opaque subtree, or claimed whole by its own marker.
    Opaque(OpaqueReport),
}

/// The entries a folder counts in the walk's summary and progress.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Counted {
    pub(crate) entries: u64,
    pub(crate) bytes: u64,
    /// Entries that could not be read.
    pub(crate) failed: u64,
}

#[derive(Debug)]
pub(crate) struct FolderReport {
    pub(crate) batch: FolderBatch,
    /// One per kept folder of the batch, in the same order.
    pub(crate) children: Vec<Child>,
    /// The listing's sibling masks, shared by every subfolder's job.
    pub(crate) child_siblings: Option<Arc<[(u32, u64)]>>,
    /// What the folder's own listing holds for its hits that name entries.
    pub(crate) held: Vec<Held>,
    pub(crate) counted: Counted,
}

#[derive(Debug)]
pub(crate) struct OpaqueReport {
    /// The folder's own marker hits, when it is the opaque root.
    pub(crate) own_hits: Vec<Hit>,
    /// The folder's own times, for its own hits.
    pub(crate) own_times: Times,
    /// What the folder's own listing holds for its hits that name entries, when it is the opaque root.
    pub(crate) held: Vec<Held>,
    /// The plain files and links, for the tree.
    pub(crate) sums: Sums,
    /// The files whose bytes may be shared, counted once by the tree.
    pub(crate) shared: Vec<SharedFile>,
    /// Every file counted once per name, as a match reports it.
    pub(crate) sizes: Sizes,
    pub(crate) folders: u32,
    /// Something in it could not be read or was not entered.
    pub(crate) incomplete: bool,
    /// What its entries hold, for the subtree's inside.
    pub(crate) inside: Inside,
    pub(crate) children: Vec<Child>,
    pub(crate) counted: Counted,
}

impl Default for OpaqueReport {
    fn default() -> Self {
        Self {
            own_hits: Vec::new(),
            own_times: Times::NONE,
            held: Vec::new(),
            sums: Sums::ZERO,
            shared: Vec::new(),
            sizes: Sizes::ZERO,
            folders: 0,
            incomplete: false,
            inside: Inside::EMPTY,
            children: Vec::new(),
            counted: Counted::default(),
        }
    }
}

/// Takes jobs until the queue ends; a cancelled token or a gone aggregator stops the walk.
pub(crate) fn run(context: Context<'_>, sender: &SyncSender<Report>) {
    let mut listing = FolderListing::new();
    while let Some(job) = context.queue.pop() {
        if context.cancel.is_cancelled() {
            context.queue.cancel();
            break;
        }
        let report = read(context, &job, &mut listing);
        if sender.send(report).is_err() {
            context.queue.cancel();
            break;
        }
    }
}

fn read(context: Context<'_>, job: &Job, listing: &mut FolderListing) -> Report {
    let body = match errors::read_folder(context.reader, &job.path, job.identity, listing) {
        Err(error) => Body::Failed(error),
        Ok(()) => folder(context, job, listing),
    };
    Report { node: job.node, opaque: job.opaque, root_device: job.root_device, body }
}

fn folder(context: Context<'_>, job: &Job, listing: &FolderListing) -> Body {
    let is_opaque_root = job.opaque.is_none_or(|root| root == job.node);
    if !is_opaque_root {
        return Body::Opaque(summed(context, job, listing, Vec::new()));
    }
    // In a place walked outside the scan roots, only the interests anchored in it match.
    let in_place = context.boundary.in_place(&job.path);
    let facts = context.matcher.folder(listing);
    let facts = if in_place { facts.without_markers() } else { facts };
    let match_entry = |name: &[u8], kind: EntryKind| {
        if in_place { context.matcher.anchored(job.state, name) } else { context.matcher.entry(job.state, name, kind) }
    };
    let own_hits = if job.is_root { Vec::new() } else { own_hits(context, job, facts.markers()) };
    let held = if job.is_root { Vec::new() } else { held(context, job, listing) };
    if job.opaque.is_some() || own_hits.iter().any(|hit| context.matcher.claims_whole(hit.interest)) {
        return Body::Opaque(OpaqueReport { held, ..summed(context, job, listing, own_hits) });
    }
    let mut hits: Vec<Hits> = Vec::with_capacity(listing.len());
    let mut children = Vec::new();
    let mut counted = Counted::default();
    for (name, entry) in listing.iter() {
        if entry.error.is_some() {
            counted.failed += 1;
            hits.push(Hits::default());
            continue;
        }
        if entry.kind != EntryKind::Folder {
            counted.entries += 1;
            if entry.kind == EntryKind::File {
                counted.bytes = counted.bytes.saturating_add(entry.sizes.logical.bytes());
            }
            hits.push(match_entry(name, entry.kind));
            continue;
        }
        let path = job.path.join(OsStr::from_bytes(name));
        let decision = context.boundary.decide(&path, entry.device, entry.access.owner, job.root_device);
        // Another user's folder in a place is listed as itself, so a module can say why it is left alone.
        let listed = matches!(decision, Decision::Enter | Decision::NotScanned(fresher_core::NotScanned::OthersFolder));
        let found = if listed { match_entry(name, entry.kind) } else { Hits::default() };
        if decision != Decision::NotScanned(fresher_core::NotScanned::OtherVolume) {
            counted.entries += 1;
        }
        let whole = found.as_slice().iter().any(|&interest| context.matcher.claims_whole(interest));
        let identity = FolderIdentity { device: entry.device, file_id: entry.file_id };
        let state = context.matcher.child_state(job.state, name);
        children.push(Child { path, identity, state, decision, whole, times: Times::of(entry) });
        hits.push(found);
    }
    let mut batch = fold(job.node, listing, &facts, &hits, context.threshold, context.user);
    batch.own_hits = own_hits;
    batch.own_times = job.times;
    let child_siblings = (!facts.siblings().is_empty()).then(|| Arc::from(facts.siblings()));
    Body::Folder(FolderReport { batch, children, child_siblings, held, counted })
}

/// The folder's own marker hits: their sibling masks come from the parent's listing; a `CACHEDIR.TAG` is read.
fn own_hits(context: Context<'_>, job: &Job, markers: &[u32]) -> Vec<Hit> {
    let siblings = job.parent_siblings.as_deref().unwrap_or_default();
    markers
        .iter()
        .map(|&interest| {
            let marker = context.matcher.checks_cachedir_tag(interest).then(|| tag_check(context, job));
            Hit { interest, marker, siblings: sibling_mask(siblings, interest), holds: 0 }
        })
        .collect()
}

/// What the folder's own listing holds for the anchored interests that end at it and name entries; a held
/// `CACHEDIR.TAG` is read once, whatever the number of interests naming it.
fn held(context: Context<'_>, job: &Job, listing: &FolderListing) -> Vec<Held> {
    let mut held = context.matcher.held(job.state, listing);
    let mut tag = None;
    for found in &mut held {
        let Some(bit) = context.matcher.held_tag(found.interest) else { continue };
        found.marker = Some(if found.holds & (1 << bit) == 0 {
            MarkerCheck::Invalid
        } else {
            *tag.get_or_insert_with(|| tag_check(context, job))
        });
    }
    held
}

/// Whether the folder's `CACHEDIR.TAG` starts with the signature; anything that cannot be read is not valid.
fn tag_check(context: Context<'_>, job: &Job) -> MarkerCheck {
    match context.reader.head(&job.path, CACHEDIR_TAG, CACHEDIR_SIGNATURE.len()) {
        Ok(head) if is_valid_cachedir_tag(&head) => MarkerCheck::Valid,
        _ => MarkerCheck::Invalid,
    }
}

/// Sums a folder inside an opaque subtree: nothing in it is matched or kept; every entry joins the subtree's inside,
/// and a folder not entered makes it incomplete.
fn summed(context: Context<'_>, job: &Job, listing: &FolderListing, own_hits: Vec<Hit>) -> OpaqueReport {
    let mut report = OpaqueReport { own_hits, own_times: job.times, ..OpaqueReport::default() };
    for (name, entry) in listing.iter() {
        report.inside.join(entry, context.user);
        report.inside.join_name(name);
        if entry.error.is_some() {
            report.incomplete = true;
            continue;
        }
        report.counted.entries += 1;
        match entry.kind {
            EntryKind::Folder => {
                let path = job.path.join(OsStr::from_bytes(name));
                let decision = context.boundary.decide(&path, entry.device, entry.access.owner, job.root_device);
                if decision != Decision::Enter {
                    report.inside.complete = false;
                }
                match decision {
                    Decision::NotScanned(fresher_core::NotScanned::OtherVolume) => continue,
                    Decision::Enter => {}
                    Decision::NotScanned(_) | Decision::Protected => report.incomplete = true,
                }
                report.folders = report.folders.saturating_add(1);
                let identity = FolderIdentity { device: entry.device, file_id: entry.file_id };
                let times = Times::NONE;
                report.children.push(Child { path, identity, state: TrieState::DEAD, decision, whole: false, times });
            }
            EntryKind::File => {
                report.counted.bytes = report.counted.bytes.saturating_add(entry.sizes.logical.bytes());
                report.sizes = report.sizes.checked_add(entry.sizes).unwrap_or_else(|| {
                    report.incomplete = true;
                    report.sizes
                });
                if entry.links > 1 || entry.clone.is_some() {
                    report.shared.push(SharedFile::of(entry));
                } else {
                    report.sums = report.sums.saturating_add(Sums::file(entry.sizes));
                }
            }
            EntryKind::Symlink | EntryKind::Other => report.sums = report.sums.saturating_add(Sums::file(Sizes::ZERO)),
        }
    }
    report
}
