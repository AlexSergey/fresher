//! Entry points for the crate's own benches (`benches/scan.rs`): they reach crate-private parts. Not part of the API.

use std::path::Path;
use std::sync::PoisonError;

use fresher_core::{
    CancelToken, DiskState, DiskWalk, EntryKind, InterestSet, StateId, Timestamp, WalkError, WalkMode, WalkRequest,
};

use crate::codec::decode::decode;
use crate::codec::encode::encode;
use crate::codec::fit::{Limits, fit, packed_len};
use crate::codec::header::Header;
use crate::scanner::{Scanner, header};
use crate::state::TreeState;
use crate::tree::arena::Tree;

use crate::matcher::Matcher;
use crate::matcher::compile::under_home;
use crate::matcher::state::TrieState;

/// A matcher compiled for one folder, as a worker holds it while it matches the folder's entries.
#[derive(Debug)]
pub struct MatcherBench {
    matcher: Matcher,
    state: TrieState,
}

impl MatcherBench {
    /// `interests` compiled with `home` as the anchor, standing at `folder`.
    pub fn new(interests: &InterestSet, home: &Path, folder: &Path) -> Self {
        let matcher = Matcher::compile(interests, &under_home(home));
        let state = matcher.root_state(folder);
        Self { matcher, state }
    }

    /// The number of interests the entry `name` of `kind` matches.
    pub fn entry(&self, name: &[u8], kind: EntryKind) -> usize {
        self.matcher.entry(self.state, name, kind).as_slice().len()
    }
}

/// The cache body of one scanned tree: encoded, read back and fitted, as the scanner's commit and rescan do.
#[derive(Debug)]
pub struct CacheBench {
    tree: Tree,
    header: Header,
}

/// The sizes of one cache body.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CacheSizes {
    pub nodes: usize,
    pub body: usize,
    pub packed: u64,
}

impl CacheBench {
    /// Walks `request` in full with `scanner` and keeps its tree, uncommitted.
    pub fn of(scanner: &Scanner, request: &WalkRequest<'_>) -> Result<Self, WalkError> {
        let full = WalkRequest { mode: WalkMode::Full, ..*request };
        scanner.walk(&full, &CancelToken::new(), &mut |_| {}, &mut |_| {})?;
        let walked = scanner.last.lock().unwrap_or_else(PoisonError::into_inner).take();
        let walked = walked.ok_or(WalkError::Commit { kind: std::io::ErrorKind::NotFound })?;
        let header = header(&walked.scope, &walked.positions, Timestamp::from_secs(0), &[]);
        Ok(Self { tree: walked.tree, header })
    }

    /// The body, uncollapsed.
    pub fn encode(&self) -> Vec<u8> {
        encode(&self.tree, &self.header).unwrap_or_default()
    }

    /// Reads `body` back; the number of nodes of the tree, 0 when it is refused.
    pub fn decode(body: &[u8]) -> usize {
        decode(body).map_or(0, |(tree, _)| tree.len())
    }

    /// The body fitted to a file of `file` bytes and a body of `body` bytes; empty when it does not fit.
    pub fn fit(&self, file: u64, body: u64) -> Vec<u8> {
        fit(&self.tree, &self.header, Limits { file, body }).unwrap_or_default()
    }

    /// The nodes, the body and its length once compressed.
    pub fn sizes(&self, body: &[u8]) -> CacheSizes {
        CacheSizes { nodes: self.tree.len(), body: body.len(), packed: packed_len(body) }
    }
}

/// The read queries of one scanned tree: the largest files and search by name.
#[derive(Debug)]
pub struct QueryBench {
    state: TreeState,
}

impl QueryBench {
    /// Walks `request` in full with `scanner` and keeps its tree as a state, as a commit without claims does.
    pub fn of(scanner: &Scanner, request: &WalkRequest<'_>) -> Result<Self, WalkError> {
        let full = WalkRequest { mode: WalkMode::Full, ..*request };
        scanner.walk(&full, &CancelToken::new(), &mut |_| {}, &mut |_| {})?;
        let walked = scanner.last.lock().unwrap_or_else(PoisonError::into_inner).take();
        let walked = walked.ok_or(WalkError::Commit { kind: std::io::ErrorKind::NotFound })?;
        Ok(Self { state: TreeState::new(StateId::new(1), Timestamp::from_secs(0), walked.tree, Vec::new()) })
    }

    /// The number of largest files answered for `limit`.
    pub fn largest_files(&self, limit: u32) -> usize {
        self.state.largest_files(limit).len()
    }

    /// The number of nodes found for `query`, at most `limit`.
    pub fn search(&self, query: &str, limit: u32) -> usize {
        self.state.search(query, limit).len()
    }

    /// The nodes of the tree.
    pub fn nodes(&self) -> usize {
        self.state.tree().len()
    }

    /// The bytes the state holds on the heap: its tree, and the lowercase copy of the names once searched.
    pub fn heap_bytes(&self) -> usize {
        self.state.heap_bytes()
    }
}

/// The folders of the last walk of `scanner`, not committed yet, that a rescan can read again (not opaque, not
/// collapsed, read): the bench scripts its journal with them.
pub fn walked_folders(scanner: &Scanner) -> Vec<std::path::PathBuf> {
    use crate::tree::node::{Flag, NodeIndex};

    let last = scanner.last.lock().unwrap_or_else(PoisonError::into_inner);
    let Some(walked) = last.as_ref() else { return Vec::new() };
    let tree = &walked.tree;
    (0..tree.len())
        .filter_map(NodeIndex::new)
        .filter(|&index| {
            tree.node(index).is_some_and(|node| {
                node.flags.has(Flag::Folder)
                    && !node.flags.has(Flag::Opaque)
                    && !node.flags.has(Flag::Collapsed)
                    && !node.flags.is_incomplete()
            })
        })
        .filter_map(|index| tree.path(index))
        .collect()
}
