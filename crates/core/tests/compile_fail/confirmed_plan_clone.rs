//! A confirmed plan cannot be cloned, so one confirmation cannot run twice.

use fresher_core::ConfirmedPlan;

fn twice(plan: ConfirmedPlan) -> (ConfirmedPlan, ConfirmedPlan) {
    (plan.clone(), plan)
}

fn main() {
    let _ = twice;
}
