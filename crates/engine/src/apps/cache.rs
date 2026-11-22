//! The bundle facts of the last committed state's listed apps: measured on the first listing (or page) after a
//! commit, in parallel, outside every engine lock, and kept until another state is committed.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

use fresher_core::{
    BundleFacts, CancelToken, CommandError, EngineStatus, InstalledApp, InstalledApps, Operation, StateId, SystemError,
};

use crate::engine::Shared;
use crate::error::lock;

/// The most threads that measure bundles at once.
const THREADS: usize = 4;

/// The facts of each installed app's bundle, by its position in the installed list; `None` for an app not measured.
pub(crate) type Facts = Vec<Option<Result<BundleFacts, SystemError>>>;

/// The facts measured for one state.
#[derive(Debug, Default)]
pub(crate) struct AppsCache {
    /// The state the facts are of; `None` before any measure.
    state: Option<StateId>,
    facts: Facts,
    /// The token of the measure running now: a scan that starts cancels it.
    measure: Option<CancelToken>,
}

impl AppsCache {
    /// Cancels the measure running now, if any: its result is not kept.
    pub(crate) fn cancel_measure(&mut self) {
        if let Some(measure) = self.measure.take() {
            measure.cancel();
        }
    }
}

/// The facts of the apps at `listed` of `installed`, the list of `state`: the kept ones when they are of `state`, else
/// measured now with no engine lock held, and kept when `state` is still the last committed one. One measure at a
/// time: a listing that waited for another takes what it kept. While a scan runs nothing is measured, and a scan that
/// starts cancels the measure: both answer `Busy`.
pub(crate) fn facts(
    shared: &Shared,
    state: StateId,
    installed: &[InstalledApp],
    listed: &[usize],
    scanning: bool,
) -> Result<Facts, CommandError> {
    let busy = || CommandError::Busy { running: Operation::Scan };
    if let Some(facts) = kept(shared, state)? {
        return Ok(facts);
    }
    if scanning {
        return Err(busy());
    }
    // Not an engine lock: only a listing waits on it, never a scan or a command of another kind.
    let _gate = lock(&shared.apps_gate)?;
    if let Some(facts) = kept(shared, state)? {
        return Ok(facts);
    }
    let cancel = CancelToken::new();
    lock(&shared.apps)?.measure = Some(cancel.clone());
    // A scan that started before the token was kept did not cancel it: it is seen here.
    if matches!(lock(&shared.machine)?.status(), EngineStatus::Scanning { .. }) {
        lock(&shared.apps)?.measure = None;
        return Err(busy());
    }
    let measured = measure(&*shared.ports.system.apps, installed, listed, &cancel);
    let current = lock(&shared.results)?.state().map(|kept| kept.id());
    let mut cache = lock(&shared.apps)?;
    cache.measure = None;
    if cancel.is_cancelled() {
        return Err(busy());
    }
    if current == Some(state) {
        cache.state = Some(state);
        cache.facts.clone_from(&measured);
    }
    Ok(measured)
}

/// The facts kept for `state`, if they are of it.
pub(super) fn kept(shared: &Shared, state: StateId) -> Result<Option<Facts>, CommandError> {
    let cache = lock(&shared.apps)?;
    Ok((cache.state == Some(state)).then(|| cache.facts.clone()))
}

/// The facts of the bundles at `listed`, at most four at once. An app whose measure did not end (its thread could not
/// start or panicked) stays `None`.
fn measure(apps: &dyn InstalledApps, installed: &[InstalledApp], listed: &[usize], cancel: &CancelToken) -> Facts {
    let next = AtomicUsize::new(0);
    let work = || {
        let mut done = Vec::new();
        while let Some(&at) = listed.get(next.fetch_add(1, Ordering::Relaxed)) {
            if cancel.is_cancelled() {
                break;
            }
            if let Some(app) = installed.get(at) {
                done.push((at, apps.bundle(app, cancel)));
            }
        }
        done
    };
    let mut facts: Facts = vec![None; installed.len()];
    thread::scope(|scope| {
        let workers: Vec<_> = (0..THREADS.min(listed.len()))
            .filter_map(|_| thread::Builder::new().name("fresher-apps".into()).spawn_scoped(scope, work).ok())
            .collect();
        for worker in workers {
            let Ok(done) = worker.join() else {
                tracing::error!("a bundle's measure panicked");
                continue;
            };
            for (at, answer) in done {
                if let Some(slot) = facts.get_mut(at) {
                    *slot = Some(answer);
                }
            }
        }
    });
    facts
}

#[cfg(test)]
#[path = "cache_tests.rs"]
mod tests;
