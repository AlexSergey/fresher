//! The consent to delete permanently cannot be made outside the confirmation.

use fresher_core::{ConfirmedMethod, PermanentConsent};

fn main() {
    let _ = ConfirmedMethod::Permanent(PermanentConsent(()));
}
