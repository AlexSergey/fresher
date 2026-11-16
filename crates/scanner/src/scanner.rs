//! `Scanner`: the core's `DiskWalk` over the full walk.

use std::fmt;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::SystemTime;

use fresher_core::{
    ByteSize, CancelToken, CategoryId, ChangeJournal, ClaimedPath, Clock, DiskState, DiskWalk, FolderReader,
    FullScanReason, JournalPosition, Match, Pattern, ScanCache, ScanKind, StateId, Timestamp, VolumeId, Volumes,
    WalkError, WalkMode, WalkProgress, WalkRequest, WalkSummary,
};

use crate::codec::fit::{Limits, fit};
use crate::codec::header::{Header, RootRecord};
use crate::matcher::Matcher;
use crate::matcher::trie::Segment;
use crate::rescan::change_set::RESCAN_LIMIT_PERCENT;
use crate::rescan::emit::emit;
use crate::rescan::load::{Cached, Use};
use crate::rescan::summary::summarize;
use crate::rescan::{Scope, ScopeRoot, Sources, Stop, rescan};
use crate::state::TreeState;
use crate::tree::arena::Tree;
use crate::tree::fold::KEEP_THRESHOLD;
use crate::tree::node::{Node, NodeIndex};
use crate::walk::Walk;
use crate::walk::boundary::Boundary;
use crate::walk::roots::Roots;

/// The most worker threads: more than the cores made the walk slower in plan 004/01.
const MAX_THREADS: usize = 16;

/// Resolves an anchored pattern to the absolute folder it is relative to; `None` when it has no anchor.
pub type AnchorResolver = Arc<dyn Fn(&Pattern) -> Option<PathBuf> + Send + Sync>;

/// The resolver of today's only anchor: `Pattern::UnderHome` against `home`, canonicalized once here (so it agrees
/// with the canonical roots), as given when it cannot be.
pub fn home_anchor(home: &Path) -> AnchorResolver {
    let home = std::fs::canonicalize(home).unwrap_or_else(|_| home.to_path_buf());
    Arc::new(move |pattern| match pattern {
        Pattern::UnderHome(_) => Some(home.clone()),
        Pattern::FolderName(_) | Pattern::MarkerFile(_) | Pattern::Extension(_) => None,
    })
}

/// How the scanner walks.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScannerOptions {
    /// The worker threads.
    pub threads: NonZeroUsize,
    /// A file whose allocated size is at least this many bytes is kept as its own record; smaller ones are folded.
    pub fold_threshold: u64,
    /// Places no walk opens, added to every request's own `avoid`: the app's stand-in until plan 006 feeds the
    /// request (docs/hacks/, "Places that prompt are avoided until plan 006").
    pub standing_avoid: Vec<PathBuf>,
    /// The largest cache body, before compression, the cache keeps: the store's body limit, passed where the app
    /// composes the store and the scanner (the port carries only the cap of the file).
    pub cache_body_limit: ByteSize,
    /// The most folders a rescan reads again, in percent of the cached tree's folders; past it the scan is full.
    pub rescan_limit_percent: u64,
}

/// The store's default body limit (64 MiB), for a scanner composed without one.
const CACHE_BODY_LIMIT: ByteSize = ByteSize::new(64 * 1024 * 1024);

impl Default for ScannerOptions {
    /// One thread per core, at most 16 (plan 004/01); files of 1 MiB and up kept.
    fn default() -> Self {
        let cores = std::thread::available_parallelism().map_or(1, NonZeroUsize::get);
        let threads = NonZeroUsize::new(cores.min(MAX_THREADS)).unwrap_or(NonZeroUsize::MIN);
        Self {
            threads,
            fold_threshold: KEEP_THRESHOLD,
            standing_avoid: Vec::new(),
            cache_body_limit: CACHE_BODY_LIMIT,
            rescan_limit_percent: RESCAN_LIMIT_PERCENT,
        }
    }
}

/// The wall clock of the system: when a state was taken.
#[derive(Debug)]
struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> SystemTime {
        SystemTime::now()
    }
}

/// The tree of a walk, until it is committed, with what it is of.
#[derive(Debug)]
pub(crate) struct Walked {
    pub(crate) tree: Tree,
    pub(crate) scope: Scope,
    /// Where each root's journal stood before the walk read anything; `None` for a volume with no journal.
    pub(crate) positions: Vec<Option<JournalPosition>>,
    /// Whether `commit` saves the tree to the cache: a restored tree is the cache's current generation already.
    save: bool,
    /// When the restored state was taken; `None` for a walk, dated when it is committed.
    taken: Option<Timestamp>,
}

/// What a walk is made of before it reads anything.
#[derive(Debug)]
pub(crate) struct Prepared {
    pub(crate) roots: Roots,
    pub(crate) boundary: Boundary,
    pub(crate) matcher: Matcher,
    pub(crate) scope: Scope,
    /// The user the entries' access is read for: the owner of the home folder; `None` (owns nothing) when it cannot
    /// be read.
    pub(crate) user: Option<u32>,
}

/// The fast scan: a parallel walk with a compiled matcher, keeping a compact tree of the last walk for `commit`, and
/// the cache of the last committed tree.
pub struct Scanner {
    reader: Arc<dyn FolderReader>,
    volumes: Arc<dyn Volumes>,
    journal: Arc<dyn ChangeJournal>,
    cache: Arc<dyn ScanCache>,
    anchor: AnchorResolver,
    options: ScannerOptions,
    clock: Arc<dyn Clock>,
    /// The tree of the last walk, until it is committed.
    pub(crate) last: Mutex<Option<Walked>>,
    commits: AtomicU32,
}

impl fmt::Debug for Scanner {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("Scanner").field("options", &self.options).finish_non_exhaustive()
    }
}

impl Scanner {
    /// A scanner reading with `reader`, keeping its last tree in `cache` and asking `journal` what changed since.
    pub fn new(
        reader: Arc<dyn FolderReader>,
        volumes: Arc<dyn Volumes>,
        journal: Arc<dyn ChangeJournal>,
        cache: Arc<dyn ScanCache>,
        anchor: AnchorResolver,
        options: ScannerOptions,
    ) -> Self {
        Self {
            reader,
            volumes,
            journal,
            cache,
            anchor,
            options,
            clock: Arc::new(SystemClock),
            last: Mutex::new(None),
            commits: AtomicU32::new(0),
        }
    }

    /// This scanner, dating its states with `clock` instead of the system's.
    #[must_use]
    pub fn with_clock(mut self, clock: Arc<dyn Clock>) -> Self {
        self.clock = clock;
        self
    }

    /// A root as the settings write it, `~` and `~/…` filled in with the home folder (the anchor of home-relative
    /// paths); any other root as given.
    fn expand_home(&self, root: &Path) -> PathBuf {
        let mut parts = root.components();
        if parts.next().map(std::path::Component::as_os_str) != Some(std::ffi::OsStr::new("~")) {
            return root.to_path_buf();
        }
        match (self.anchor)(&Pattern::UnderHome(String::new())) {
            Some(home) => home.join(parts.as_path()),
            None => root.to_path_buf(),
        }
    }

    fn compile(&self, request: &WalkRequest<'_>) -> Matcher {
        let anchor = |pattern: &Pattern| {
            let Pattern::UnderHome(relative) = pattern else { return None };
            (self.anchor)(pattern).map(|at| (at, Segment::parse(relative)))
        };
        Matcher::compile(request.interests, &anchor)
    }

    /// The roots, the boundaries, the matcher and the scope of `request`; reads no folder.
    pub(crate) fn prepare(&self, request: &WalkRequest<'_>) -> Result<Prepared, WalkError> {
        let asked: Vec<PathBuf> = request.roots.iter().map(|root| self.expand_home(root)).collect();
        let roots = Roots::prepare(&asked, &*self.volumes, &*self.reader)?;
        let canonical: Vec<PathBuf> = roots.iter().map(|root| root.path.clone()).collect();
        let avoid: Vec<PathBuf> = request.avoid.iter().chain(&self.options.standing_avoid).cloned().collect();
        let boundary = Boundary::new(&canonical, &avoid);
        let matcher = self.compile(request);
        let scope = Self::scope(&roots, &avoid, &matcher);
        Ok(Prepared { roots, boundary, matcher, scope, user: self.user() })
    }

    /// The user the scan is for: the owner of the home folder, read through the reader. The app runs as the user whose
    /// home it scans; without a home folder, or one that cannot be read, no user is known and nothing is owned.
    fn user(&self) -> Option<u32> {
        let home = (self.anchor)(&Pattern::UnderHome(String::new()))?;
        match self.reader.access(&home) {
            Ok(access) => access.owner,
            Err(error) => {
                tracing::warn!(%error, "the owner of the home folder is unknown: nothing is shown as removable");
                None
            }
        }
    }

    /// The cached state of the request's scope, its matches emitted, read from the cache alone: no folder is read,
    /// the journal is not asked. Any reason the cache cannot be used is `NothingToRestore` (logged).
    fn restore(&self, request: &WalkRequest<'_>, on_match: &mut dyn FnMut(Match)) -> Result<WalkSummary, WalkError> {
        let asked: Vec<PathBuf> = request.roots.iter().map(|root| self.expand_home(root)).collect();
        let roots = Roots::locate(&asked, &*self.volumes).map_err(|error| {
            tracing::info!(%error, "a scan root cannot be restored");
            WalkError::NothingToRestore
        })?;
        let avoid: Vec<PathBuf> = request.avoid.iter().chain(&self.options.standing_avoid).cloned().collect();
        let matcher = self.compile(request);
        let scope = Self::scope(&roots, &avoid, &matcher);
        let Cached { tree, header } = Cached::load(&*self.cache, &scope, Use::Restore).map_err(|reason| {
            tracing::info!(?reason, "the cache cannot be restored");
            WalkError::NothingToRestore
        })?;
        emit(&tree, &matcher, request.interests, on_match);
        let summary = summarize(&tree, ScanKind::Restored { taken: header.taken });
        let positions = header.roots.iter().map(|root| root.journal).collect();
        *self.last.lock().unwrap_or_else(PoisonError::into_inner) =
            Some(Walked { tree, scope, positions, save: false, taken: Some(header.taken) });
        Ok(summary)
    }

    /// What a walk of `roots` avoiding `avoid` with `matcher` is of.
    fn scope(roots: &Roots, avoid: &[PathBuf], matcher: &Matcher) -> Scope {
        let mut avoid = avoid.to_vec();
        avoid.sort();
        avoid.dedup();
        Scope {
            key: roots.scope_key(),
            roots: roots
                .iter()
                .map(|root| ScopeRoot { path: root.path.clone(), volume: root.volume.id, identity: root.identity })
                .collect(),
            avoid,
            fingerprint: matcher.fingerprint(),
        }
    }

    /// Where the journal of each root's volume stands now, asked once per volume.
    fn positions(&self, scope: &Scope) -> Vec<Option<JournalPosition>> {
        let mut asked: Vec<(VolumeId, Option<JournalPosition>)> = Vec::new();
        scope
            .roots
            .iter()
            .map(|root| {
                if let Some(&(_, position)) = asked.iter().find(|(volume, _)| *volume == root.volume) {
                    return position;
                }
                let position = self.journal.position(&root.volume);
                asked.push((root.volume, position));
                position
            })
            .collect()
    }

    /// Writes the committed `state` of `scope` to the cache within its cap. A tree that does not fit, or a cache that
    /// refuses it, is logged and not saved: the cache is safe to lose.
    fn save(&self, scope: &Scope, positions: &[Option<JournalPosition>], state: &TreeState) {
        let header = header(scope, positions, state.taken(), state.owners());
        let limits = Limits { file: self.cache.cap().bytes(), body: self.options.cache_body_limit.bytes() };
        let Ok(body) = fit(state.tree(), &header, limits) else { return };
        if let Err(error) = self.cache.save(scope.key, &body) {
            tracing::warn!(%error, "the scan cache could not be saved");
        }
    }
}

/// The head of the cache body of a tree of `scope`, taken at `taken` with the journal at `positions`.
pub(crate) fn header(
    scope: &Scope,
    positions: &[Option<JournalPosition>],
    taken: Timestamp,
    owners: &[CategoryId],
) -> Header {
    Header {
        taken,
        roots: scope
            .roots
            .iter()
            .zip(positions)
            .map(|(root, &journal)| RootRecord { path: root.path.clone(), volume: root.volume, journal })
            .collect(),
        avoid: scope.avoid.clone(),
        fingerprint: scope.fingerprint,
        claims: owners.iter().map(ToString::to_string).collect(),
    }
}

impl DiskWalk for Scanner {
    /// `Full` reads every folder; `Auto` patches the cached tree with the folders the journal says changed, and reads
    /// every folder on any doubt (`fallback` says why); `Restore` reads no folder: the cached state is emitted.
    fn walk(
        &self,
        request: &WalkRequest<'_>,
        cancel: &CancelToken,
        on_match: &mut dyn FnMut(Match),
        on_progress: &mut dyn FnMut(WalkProgress),
    ) -> Result<WalkSummary, WalkError> {
        if request.mode == WalkMode::Restore {
            return self.restore(request, on_match);
        }
        let Prepared { roots, boundary, matcher, scope, user } = self.prepare(request)?;
        let positions = self.positions(&scope);
        let walk = Walk {
            roots: &roots,
            interests: request.interests,
            matcher: &matcher,
            boundary: &boundary,
            reader: &*self.reader,
            threads: self.options.threads.get(),
            threshold: self.options.fold_threshold,
            user,
        };
        let mut fallback = (request.mode == WalkMode::Full).then_some(FullScanReason::AskedByUser);
        if request.mode == WalkMode::Auto {
            let now = Timestamp::from_system_time(self.clock.now());
            let sources = Sources {
                cache: &*self.cache,
                journal: &*self.journal,
                now,
                seed: now.secs(),
                limit_percent: self.options.rescan_limit_percent,
            };
            match rescan(&scope, &walk, sources, cancel, on_match, on_progress) {
                Ok((tree, summary)) => {
                    *self.last.lock().unwrap_or_else(PoisonError::into_inner) =
                        Some(Walked { tree, scope, positions, save: true, taken: None });
                    return Ok(summary);
                }
                Err(Stop::Walk(error)) => return Err(error),
                Err(Stop::Full(reason)) => {
                    tracing::info!(?reason, "the rescan becomes a full scan");
                    fallback = Some(reason);
                }
            }
        }
        let (tree, mut summary) = walk.run(cancel, on_match, on_progress)?;
        summary.fallback = fallback;
        *self.last.lock().unwrap_or_else(PoisonError::into_inner) =
            Some(Walked { tree, scope, positions, save: true, taken: None });
        Ok(summary)
    }

    /// Records each claimed path's category on its node and keeps the state. A claim on a path that is not a node
    /// (inside a subtree claimed whole, say) is left out. Without a walk to commit: `Commit { NotFound }`.
    fn commit(&self, claims: &[ClaimedPath]) -> Result<Arc<dyn DiskState>, WalkError> {
        let Some(Walked { mut tree, scope, positions, save, taken }) =
            self.last.lock().unwrap_or_else(PoisonError::into_inner).take()
        else {
            return Err(WalkError::Commit { kind: std::io::ErrorKind::NotFound });
        };
        let refused = |_| WalkError::Commit { kind: std::io::ErrorKind::NotFound };
        for index in (0..tree.len()).filter_map(NodeIndex::new) {
            tree.set_owner(index, Node::NO_OWNER).map_err(refused)?;
        }
        let mut owners: Vec<CategoryId> = Vec::new();
        for claim in claims {
            let Some(node) = tree.lookup(&claim.path) else { continue };
            let owner = owners.iter().position(|&known| known == claim.category).unwrap_or_else(|| {
                owners.push(claim.category);
                owners.len() - 1
            });
            if let Ok(owner) = u32::try_from(owner) {
                tree.set_owner(node, owner).map_err(refused)?;
            }
        }
        let id = StateId::new(self.commits.fetch_add(1, Ordering::Relaxed).saturating_add(1));
        let taken = taken.unwrap_or_else(|| Timestamp::from_system_time(self.clock.now()));
        let state = Arc::new(TreeState::new(id, taken, tree, owners));
        if save {
            self.save(&scope, &positions, &state);
        }
        Ok(state)
    }
}
