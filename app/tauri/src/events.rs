//! The one thread that forwards the engine's events to the webview. A reloaded webview subscribes again with a new
//! channel; the new channel replaces the old one, so the app never runs more than one forwarding thread.

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::thread::{self, JoinHandle};

use fresher_core::{CommandError, Event};
use fresher_engine::Engine;
use tauri::ipc::Channel;

/// The channel the events go to, shared with the forwarding thread.
type Target = Arc<Mutex<Option<Channel<Event>>>>;

/// The forwarding thread and its channel; the app manages one.
#[derive(Default)]
pub(crate) struct EventForwarder {
    target: Target,
    thread: Mutex<Option<JoinHandle<()>>>,
}

impl EventForwarder {
    /// Sends every event of `engine` from now on to `channel` instead of the channel before it. The thread starts on
    /// the first call and lives as long as the engine's events do.
    pub(crate) fn forward(&self, engine: &Engine, channel: Channel<Event>) -> Result<(), CommandError> {
        let mut thread = lock(&self.thread);
        *lock(&self.target) = Some(channel);
        if thread.is_none() {
            let events = engine.subscribe();
            let target = Arc::clone(&self.target);
            let handle = thread::Builder::new()
                .name("fresher-events".into())
                .spawn(move || {
                    for event in events {
                        let mut channel = lock(&target);
                        // A channel the webview no longer listens on is dropped; the next subscribe brings a new one.
                        if channel.as_ref().is_some_and(|current| current.send(event).is_err()) {
                            *channel = None;
                        }
                    }
                })
                .map_err(|_| CommandError::Internal)?;
            *thread = Some(handle);
        }
        Ok(())
    }

    /// The forwarding thread, if it was started.
    #[cfg(test)]
    pub(crate) fn thread_id(&self) -> Option<thread::ThreadId> {
        lock(&self.thread).as_ref().map(|handle| handle.thread().id())
    }
}

/// The lock stays usable whatever a panicking thread was doing with it.
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}
