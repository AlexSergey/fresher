//! A confirmation cannot be made from the plan it confirms, nor by default: no `From`, no builder, no `Default`.

use fresher_core::{CleanupPlan, Confirmation};

fn echo_of(plan: &CleanupPlan) -> Confirmation {
    Confirmation::from(plan)
}

fn main() {
    let _ = echo_of;
    let _ = Confirmation::default();
}
