//! Loading the cached tree of a scope and checking that it is of the scan asked for.

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

impl Cached {
    /// The current generation of `scope` in `cache`, checked against `scope`: to read again only what changed since,
    /// it must be of the same walk, avoid list and journal positions included. Anything that does not hold is the
    /// reason for a full scan; nothing here reads a folder.
    pub(crate) fn load(cache: &dyn ScanCache, scope: &Scope) -> Result<Self, FullScanReason> {
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
        check(&tree, &header, scope)?;
        Ok(Self { tree, header })
    }
}

/// Whether the cache is of `scope`: a rescan of another avoid list would read the changes of another walk.
fn check(tree: &Tree, header: &Header, scope: &Scope) -> Result<(), FullScanReason> {
    let same_paths = header.roots.iter().map(|root| &root.path).eq(scope.roots.iter().map(|root| &root.path));
    if !same_paths || tree.roots().len() != header.roots.len() {
        return Err(FullScanReason::OtherRoots);
    }
    if header.roots.iter().zip(&scope.roots).any(|(cached, asked)| cached.volume != asked.volume) {
        return Err(FullScanReason::OtherVolume);
    }
    if header.avoid != scope.avoid {
        return Err(FullScanReason::OtherAvoidList);
    }
    if header.fingerprint != scope.fingerprint {
        return Err(FullScanReason::OtherFingerprint);
    }
    if header.roots.iter().any(|root| root.journal.is_none()) {
        return Err(FullScanReason::JournalGap(JournalGap::NoJournal));
    }
    if tree.roots().iter().zip(&scope.roots).any(|(cached, asked)| cached.identity != asked.identity) {
        return Err(FullScanReason::JournalGap(JournalGap::RootChanged));
    }
    Ok(())
}
