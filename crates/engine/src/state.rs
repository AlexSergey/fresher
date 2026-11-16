//! The engine's lifecycle as a state machine: `Idle → Scanning → Idle` and `Idle → Cleaning → Idle`. One operation
//! at a time.

use std::sync::mpsc::Receiver;
use std::thread::JoinHandle;

use fresher_core::{CancelToken, CleanupId, CommandError, EngineStatus, Operation, ScanId};

use crate::error::EngineError;

/// What the engine is doing now.
#[derive(Debug)]
pub(crate) enum State {
    Idle,
    Scanning(ScanId, Running),
    Cleaning(CleanupId, Running),
}

/// A running operation: how to stop it, and its thread (none in the tests of the transitions).
#[derive(Debug)]
pub(crate) struct Running {
    pub(crate) cancel: CancelToken,
    pub(crate) thread: Option<OperationThread>,
}

/// The thread of a running operation, and a channel that closes when the thread ends, whether it returns or panics.
#[derive(Debug)]
pub(crate) struct OperationThread {
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
            State::Scanning(scan, _) => EngineStatus::Scanning { scan: *scan },
            State::Cleaning(cleanup, _) => EngineStatus::Cleaning { cleanup: *cleanup },
        }
    }

    /// `Ok` when no operation runs; `Busy` with the running one otherwise.
    pub(crate) const fn idle(&self) -> Result<(), CommandError> {
        match self.state {
            State::Idle => Ok(()),
            State::Scanning(..) => Err(CommandError::Busy { running: Operation::Scan }),
            State::Cleaning(..) => Err(CommandError::Busy { running: Operation::Clean }),
        }
    }

    /// The id the next scan gets, if a scan may start now.
    pub(crate) fn next_scan(&self) -> Result<ScanId, CommandError> {
        self.idle()?;
        let next = self.last_scan.checked_add(1).ok_or(EngineError::ScanIdsExhausted)?;
        Ok(ScanId::new(next))
    }

    /// `Idle → Scanning`: the scan from [`Machine::next_scan`] is running.
    pub(crate) fn start(&mut self, scan: ScanId, running: Running) {
        self.last_scan = scan.get();
        self.state = State::Scanning(scan, running);
    }

    /// `Idle → Cleaning`: the cleanup the journal numbered `cleanup` is running; the caller checked [`Machine::idle`].
    pub(crate) fn start_cleaning(&mut self, cleanup: CleanupId, running: Running) {
        self.state = State::Cleaning(cleanup, running);
    }

    /// Asks the running `operation` to stop; it ends on its own thread. `NotRunning` when another one runs or none.
    pub(crate) fn cancel(&self, operation: Operation) -> Result<(), CommandError> {
        match (&self.state, operation) {
            (State::Scanning(_, running), Operation::Scan) | (State::Cleaning(_, running), Operation::Clean) => {
                running.cancel.cancel();
                Ok(())
            }
            _ => Err(CommandError::NotRunning),
        }
    }

    /// `Scanning → Idle` when `scan` is the running one; anything else is left as it is.
    pub(crate) fn finish(&mut self, scan: ScanId) {
        if matches!(&self.state, State::Scanning(running, _) if *running == scan) {
            self.state = State::Idle;
        }
    }

    /// `Cleaning → Idle` when `cleanup` is the running one; anything else is left as it is.
    pub(crate) fn finish_cleaning(&mut self, cleanup: CleanupId) {
        if matches!(&self.state, State::Cleaning(running, _) if *running == cleanup) {
            self.state = State::Idle;
        }
    }

    /// Cancels the running operation and hands over its thread to be joined.
    pub(crate) fn stop(&mut self) -> Option<OperationThread> {
        match &mut self.state {
            State::Idle => None,
            State::Scanning(_, running) | State::Cleaning(_, running) => {
                running.cancel.cancel();
                running.thread.take()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use fresher_core::CancelToken;

    use super::Running;

    fn running() -> Running {
        Running { cancel: CancelToken::new(), thread: None }
    }

    mod negative {
        use fresher_core::{CleanupId, CommandError, EngineStatus, Operation, ScanId};

        use super::running;
        use crate::state::Machine;

        #[test]
        fn refuses_a_second_scan_while_scanning() {
            let mut machine = Machine::new();
            machine.start(ScanId::new(1), running());

            assert_eq!(machine.next_scan(), Err(CommandError::Busy { running: Operation::Scan }));
        }

        #[test]
        fn refuses_a_scan_while_cleaning() {
            let mut machine = Machine::new();
            machine.start_cleaning(CleanupId::new(1), running());

            assert_eq!(machine.next_scan(), Err(CommandError::Busy { running: Operation::Clean }));
            assert_eq!(machine.idle(), Err(CommandError::Busy { running: Operation::Clean }));
        }

        #[test]
        fn refuses_to_cancel_when_idle() {
            assert_eq!(Machine::new().cancel(Operation::Scan), Err(CommandError::NotRunning));
            assert_eq!(Machine::new().cancel(Operation::Clean), Err(CommandError::NotRunning));
        }

        #[test]
        fn refuses_to_cancel_another_operation() {
            let mut machine = Machine::new();
            machine.start_cleaning(CleanupId::new(1), running());

            assert_eq!(machine.cancel(Operation::Scan), Err(CommandError::NotRunning));
        }

        #[test]
        fn the_end_of_another_operation_changes_nothing() {
            let mut machine = Machine::new();
            machine.start(ScanId::new(2), running());

            machine.finish(ScanId::new(1));
            machine.finish_cleaning(CleanupId::new(2));

            assert_eq!(machine.status(), EngineStatus::Scanning { scan: ScanId::new(2) });
        }

        #[test]
        fn refuses_a_scan_when_the_ids_ran_out() {
            let mut machine = Machine::new();
            machine.start(ScanId::new(u32::MAX), running());
            machine.finish(ScanId::new(u32::MAX));

            assert_eq!(machine.next_scan(), Err(CommandError::Internal));
        }

        #[test]
        fn stopping_when_idle_hands_over_nothing() {
            assert!(Machine::new().stop().is_none());
        }
    }

    mod positive {
        use fresher_core::{CleanupId, EngineStatus, Operation, ScanId};

        use super::running;
        use crate::state::Machine;

        #[test]
        fn goes_idle_scanning_idle() {
            let mut machine = Machine::new();
            assert_eq!(machine.status(), EngineStatus::Idle);

            let scan = machine.next_scan().unwrap();
            machine.start(scan, running());
            assert_eq!(machine.status(), EngineStatus::Scanning { scan });

            machine.finish(scan);
            assert_eq!(machine.status(), EngineStatus::Idle);
        }

        #[test]
        fn goes_idle_cleaning_idle() {
            let mut machine = Machine::new();
            let cleanup = CleanupId::new(4);

            machine.start_cleaning(cleanup, running());
            assert_eq!(machine.status(), EngineStatus::Cleaning { cleanup });

            machine.finish_cleaning(cleanup);
            assert_eq!(machine.status(), EngineStatus::Idle);
            assert_eq!(machine.idle(), Ok(()));
        }

        #[test]
        fn scan_ids_start_at_one_and_increase() {
            let mut machine = Machine::new();

            assert_eq!(machine.next_scan(), Ok(ScanId::new(1)));
            machine.start(ScanId::new(1), running());
            machine.finish(ScanId::new(1));
            assert_eq!(machine.next_scan(), Ok(ScanId::new(2)));
        }

        #[test]
        fn cancel_and_stop_reach_the_running_operation() {
            for clean in [false, true] {
                let mut machine = Machine::new();
                let operation = running();
                let cancel = operation.cancel.clone();
                if clean {
                    machine.start_cleaning(CleanupId::new(1), operation);
                } else {
                    machine.start(ScanId::new(1), operation);
                }

                machine.cancel(if clean { Operation::Clean } else { Operation::Scan }).unwrap();
                assert!(cancel.is_cancelled());
                assert!(machine.stop().is_none());
            }
        }
    }
}
