//! A confirmed plan is executed by value: once given away, it is gone.

use fresher_core::ConfirmedPlan;

fn execute(plan: ConfirmedPlan) {
    drop(plan);
}

fn run_twice(plan: ConfirmedPlan) {
    execute(plan);
    execute(plan);
}

fn main() {
    let _ = run_twice;
}
