//! Loading the cached tree of a scope and checking that it is of the scan asked for.

use std::path::PathBuf;

use fresher_core::{FullScanReason, JournalGap, ScanCache};

use crate::codec::decode::decode;
use crate::codec::header::Header;
use crate::rescan::Scope;
use crate::tree::arena::Tree;

/// The cached tree of the scope, frozen, with what it is of.
#[derive(Debug)]
pub(crate) struct Cached {
    pub(crate) tree: Tree,
    pub(crate) header: Header,
}

/// What the cache is loaded for, which decides how much of it must match the scan asked for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Use {
    /// Read again only what changed since: the cache must be of the same walk, avoid list and journal positions
    /// included.
    Rescan,
    /// Shown as it was, reading no folder: the cache keeps the places it avoided, and the journal is not asked.
    Restore,
}

impl Cached {
    /// The current generation of `scope` in `cache`, checked against `scope`. Anything that does not hold is the
    /// reason for a full scan; nothing here reads a folder.
    pub(crate) fn load(cache: &dyn ScanCache, scope: &Scope, purpose: Use) -> Result<Self, FullScanReason> {
        let body = match cache.load(scope.key) {
            Ok(Some(body)) => body,
            Ok(None) => return Err(FullScanReason::NoCache),
            Err(error) => {
                tracing::warn!(%error, "the scan cache could not be read");
                return Err(FullScanReason::BrokenCache);
            }
        };
        let (tree, header) = decode(&body).map_err(|error| {
            tracing::warn!(%error, "the scan cache is not readable");
            FullScanReason::BrokenCache
        })?;
        check(&tree, &header, scope, purpose)?;
        Ok(Self { tree, header })
    }
}

/// Whether the cache is of `scope`. A restore compares the scan roots only: it emits nothing inside a place, so a
/// place gone missing since does not refuse it. A restore takes the cache's avoid list too: it shows that scan as it
/// was, while a rescan of another avoid list would read the changes of another walk.
fn check(tree: &Tree, header: &Header, scope: &Scope, purpose: Use) -> Result<(), FullScanReason> {
    let compared = |path: &PathBuf| purpose == Use::Rescan || !scope.places.contains(path);
    let cached: Vec<_> = header.roots.iter().filter(|root| compared(&root.path)).collect();
    let asked: Vec<_> = scope.roots.iter().filter(|root| compared(&root.path)).collect();
    let same_paths = cached.iter().map(|root| &root.path).eq(asked.iter().map(|root| &root.path));
    if !same_paths || tree.roots().len() != header.roots.len() {
        return Err(FullScanReason::OtherRoots);
    }
    if cached.iter().zip(&asked).any(|(cached, asked)| cached.volume != asked.volume) {
        return Err(FullScanReason::OtherVolume);
    }
    if purpose == Use::Rescan && header.avoid != scope.avoid {
        return Err(FullScanReason::OtherAvoidList);
    }
    if header.fingerprint != scope.fingerprint {
        return Err(FullScanReason::OtherFingerprint);
    }
    if purpose == Use::Restore {
        return Ok(());
    }
    if header.roots.iter().any(|root| root.journal.is_none()) {
        return Err(FullScanReason::JournalGap(JournalGap::NoJournal));
    }
    if tree.roots().iter().zip(&scope.roots).any(|(cached, asked)| cached.identity != asked.identity) {
        return Err(FullScanReason::JournalGap(JournalGap::RootChanged));
    }
    Ok(())
}
