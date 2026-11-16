//! The licence to act cannot be made by hand: only a confirmed plan lends one.

use fresher_core::{Act, ConfirmedPlan};

fn act_on(plan: &ConfirmedPlan) -> Act<'_> {
    Act { plan }
}

fn main() {
    let _ = act_on;
}
