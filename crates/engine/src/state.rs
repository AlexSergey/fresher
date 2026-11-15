//! The engine's lifecycle as a state machine: `Idle → Scanning → Idle`. One operation at a time.

use std::sync::mpsc::Receiver;
use std::thread::JoinHandle;

use fresher_core::{CancelToken, CommandError, EngineStatus, Operation, ScanId};

use crate::error::EngineError;

/// What the engine is doing now.
#[derive(Debug)]
pub(crate) enum State {
    Idle,
    Scanning(Running),
}

/// A running scan: its id, how to stop it, and its thread (none in the tests of the transitions).
#[derive(Debug)]
pub(crate) struct Running {
    pub(crate) scan: ScanId,
    pub(crate) cancel: CancelToken,
    pub(crate) thread: Option<ScanThread>,
}

/// The thread of a running scan, and a channel that closes when the thread ends, whether it returns or panics.
#[derive(Debug)]
pub(crate) struct ScanThread {
    pub(crate) handle: JoinHandle<()>,
    pub(crate) done: Receiver<()>,
}

/// The state and the id of the last scan started.
#[derive(Debug)]
pub(crate) struct Machine {
    state: State,
    last_scan: u32,
}

impl Machine {
    pub(crate) const fn new() -> Self {
        Self { state: State::Idle, last_scan: 0 }
    }

    pub(crate) const fn status(&self) -> EngineStatus {
        match &self.state {
            State::Idle => EngineStatus::Idle,
            State::Scanning(running) => EngineStatus::Scanning { scan: running.scan },
        }
    }

    /// The id the next scan gets, if a scan may start now.
    pub(crate) fn next_scan(&self) -> Result<ScanId, CommandError> {
        if let State::Scanning(_) = self.state {
            return Err(CommandError::Busy { running: Operation::Scan });
        }
        let next = self.last_scan.checked_add(1).ok_or(EngineError::ScanIdsExhausted)?;
        Ok(ScanId::new(next))
    }

    /// `Idle → Scanning`: the scan from [`Machine::next_scan`] is running.
    pub(crate) fn start(&mut self, running: Running) {
        self.last_scan = running.scan.get();
        self.state = State::Scanning(running);
    }

    /// Asks the running scan to stop; it ends on its own thread.
    pub(crate) fn cancel(&self) -> Result<(), CommandError> {
        match &self.state {
            State::Idle => Err(CommandError::NotRunning),
            State::Scanning(running) => {
                running.cancel.cancel();
                Ok(())
            }
        }
    }

    /// `Scanning → Idle` when `scan` is the running one; anything else is left as it is.
    pub(crate) fn finish(&mut self, scan: ScanId) {
        if matches!(&self.state, State::Scanning(running) if running.scan == scan) {
            self.state = State::Idle;
        }
    }

    /// Cancels the running scan and hands over its thread to be joined.
    pub(crate) fn stop(&mut self) -> Option<ScanThread> {
        match &mut self.state {
            State::Idle => None,
            State::Scanning(running) => {
                running.cancel.cancel();
                running.thread.take()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use fresher_core::{CancelToken, ScanId};

    use super::Running;

    fn running(scan: u32) -> Running {
        Running { scan: ScanId::new(scan), cancel: CancelToken::new(), thread: None }
    }

    mod negative {
        use fresher_core::{CommandError, EngineStatus, Operation, ScanId};

        use super::running;
        use crate::state::Machine;

        #[test]
        fn refuses_a_second_scan_while_scanning() {
            let mut machine = Machine::new();
            machine.start(running(1));

            assert_eq!(machine.next_scan(), Err(CommandError::Busy { running: Operation::Scan }));
        }

        #[test]
        fn refuses_to_cancel_when_idle() {
            assert_eq!(Machine::new().cancel(), Err(CommandError::NotRunning));
        }

        #[test]
        fn the_end_of_another_scan_changes_nothing() {
            let mut machine = Machine::new();
            machine.start(running(2));

            machine.finish(ScanId::new(1));

            assert_eq!(machine.status(), EngineStatus::Scanning { scan: ScanId::new(2) });
        }

        #[test]
        fn refuses_a_scan_when_the_ids_ran_out() {
            let mut machine = Machine::new();
            machine.start(running(u32::MAX));
            machine.finish(ScanId::new(u32::MAX));

            assert_eq!(machine.next_scan(), Err(CommandError::Internal));
        }

        #[test]
        fn stopping_when_idle_hands_over_nothing() {
            assert!(Machine::new().stop().is_none());
        }
    }

    mod positive {
        use fresher_core::{EngineStatus, ScanId};

        use super::running;
        use crate::state::Machine;

        #[test]
        fn goes_idle_scanning_idle() {
            let mut machine = Machine::new();
            assert_eq!(machine.status(), EngineStatus::Idle);

            let scan = machine.next_scan().unwrap();
            machine.start(running(scan.get()));
            assert_eq!(machine.status(), EngineStatus::Scanning { scan });

            machine.finish(scan);
            assert_eq!(machine.status(), EngineStatus::Idle);
        }

        #[test]
        fn scan_ids_start_at_one_and_increase() {
            let mut machine = Machine::new();

            assert_eq!(machine.next_scan(), Ok(ScanId::new(1)));
            machine.start(running(1));
            machine.finish(ScanId::new(1));
            assert_eq!(machine.next_scan(), Ok(ScanId::new(2)));
        }

        #[test]
        fn cancel_and_stop_reach_the_running_scan() {
            let mut machine = Machine::new();
            let scan = running(1);
            let cancel = scan.cancel.clone();
            machine.start(scan);

            machine.cancel().unwrap();
            assert!(cancel.is_cancelled());
            assert!(machine.stop().is_none());
        }
    }
}
