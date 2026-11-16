//! The cleaner executes only a confirmed plan: an unconfirmed one does not fit.

use fresher_core::{CancelToken, Cleaner, CleanupContext, CleanupPlan};

fn clean_without_asking(cleaner: &dyn Cleaner, plan: CleanupPlan, cx: &CleanupContext<'_>) {
    let _ = cleaner.execute(plan, cx, &CancelToken::new(), &mut |_| {});
}

fn main() {
    let _ = clean_without_asking;
}
