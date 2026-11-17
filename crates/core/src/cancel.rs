//! Cancellation of a running operation.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// A flag shared by every thread of one operation: once cancelled, every clone sees it. Cloning is cheap.
#[derive(Clone, Debug, Default)]
pub struct CancelToken(Arc<AtomicBool>);

impl CancelToken {
    pub fn new() -> Self {
        Self::default()
    }

    /// Asks every holder of the token to stop; there is no way back.
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Release);
    }

    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }

    /// Whether `other` is a clone of this token: the same operation. A module that keeps something between its calls
    /// of one scan tells by it a new scan from the one it kept it for.
    pub fn is_same(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

#[cfg(test)]
mod tests {
    mod negative {
        use crate::CancelToken;

        #[test]
        fn a_new_token_is_not_cancelled() {
            assert!(!CancelToken::new().is_cancelled());
        }

        #[test]
        fn two_new_tokens_are_not_the_same() {
            assert!(!CancelToken::new().is_same(&CancelToken::new()));
        }
    }

    mod positive {
        use crate::CancelToken;

        #[test]
        fn a_clone_is_the_same_token() {
            let token = CancelToken::new();

            assert!(token.is_same(&token.clone()));
        }

        #[test]
        fn cancel_token_is_seen_by_every_clone() {
            let token = CancelToken::new();
            let clones = [token.clone(), token.clone()];

            std::thread::spawn({
                let token = token.clone();
                move || token.cancel()
            })
            .join()
            .unwrap();

            assert!(token.is_cancelled());
            assert!(clones.iter().all(CancelToken::is_cancelled));
        }
    }
}
