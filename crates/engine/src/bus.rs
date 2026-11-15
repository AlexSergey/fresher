//! The event bus: every event goes to every live subscriber, in the order it was published.

use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Mutex, PoisonError};

use fresher_core::Event;

/// Fans events out to subscribers. A subscriber whose receiver is gone is dropped; nobody is ever waited for.
#[derive(Debug, Default)]
pub struct EventBus {
    subscribers: Mutex<Vec<Sender<Event>>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self::default()
    }

    /// A new receiver of every event published from now on.
    pub fn subscribe(&self) -> Receiver<Event> {
        let (sender, receiver) = mpsc::channel();
        self.lock().push(sender);
        receiver
    }

    /// Sends a copy of `event` to every live subscriber.
    pub fn publish(&self, event: &Event) {
        self.lock().retain(|subscriber| subscriber.send(event.clone()).is_ok());
    }

    /// The list of senders stays valid whatever a panicking thread was doing with it.
    fn lock(&self) -> std::sync::MutexGuard<'_, Vec<Sender<Event>>> {
        self.subscribers.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

#[cfg(test)]
mod tests {
    use fresher_core::{Event, ScanId, ScanOutcome};

    fn finished(scan: u32) -> Event {
        Event::ScanFinished { scan: ScanId::new(scan), outcome: ScanOutcome::Completed }
    }

    mod negative {
        use super::finished;
        use crate::EventBus;

        #[test]
        fn a_dropped_subscriber_does_not_stop_the_others() {
            let bus = EventBus::new();
            let gone = bus.subscribe();
            let live = bus.subscribe();
            drop(gone);

            bus.publish(&finished(1));
            bus.publish(&finished(2));

            assert_eq!(live.try_iter().collect::<Vec<_>>(), vec![finished(1), finished(2)]);
            assert_eq!(bus.lock().len(), 1);
        }

        #[test]
        fn publishing_without_subscribers_does_nothing() {
            let bus = EventBus::new();

            bus.publish(&finished(1));

            assert!(bus.lock().is_empty());
        }
    }

    mod positive {
        use super::finished;
        use crate::EventBus;

        #[test]
        fn every_subscriber_receives_every_event_in_order() {
            let bus = EventBus::new();
            let first = bus.subscribe();
            let second = bus.subscribe();

            for scan in 1..=3 {
                bus.publish(&finished(scan));
            }

            let expected = vec![finished(1), finished(2), finished(3)];
            assert_eq!(first.try_iter().collect::<Vec<_>>(), expected);
            assert_eq!(second.try_iter().collect::<Vec<_>>(), expected);
        }

        #[test]
        fn a_late_subscriber_receives_only_later_events() {
            let bus = EventBus::new();
            bus.publish(&finished(1));
            let late = bus.subscribe();

            bus.publish(&finished(2));

            assert_eq!(late.try_iter().collect::<Vec<_>>(), vec![finished(2)]);
        }
    }
}
