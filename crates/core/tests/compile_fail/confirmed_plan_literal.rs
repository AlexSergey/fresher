//! A confirmed plan cannot be written as a literal: only `CleanupPlan::confirm` makes one.

use fresher_core::{CleanupPlan, ConfirmedMethod, ConfirmedPlan};

fn confirm_without_the_user(plan: CleanupPlan) -> ConfirmedPlan {
    ConfirmedPlan { plan, method: ConfirmedMethod::Trash }
}

fn main() {
    let _ = confirm_without_the_user;
}
