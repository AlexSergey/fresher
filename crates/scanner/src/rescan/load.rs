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

/// Whether the journal positions are asked of the cache: a rescan needs them, a restore does not.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Use {
    Rescan,
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
/// place gone missing since does not refuse it.
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
    if header.avoid != scope.avoid {
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
