//! A checked item cannot be copied: one check serves one act.

use fresher_cleaner::Verified;

fn copy<T: Clone>(item: &T) -> T {
    item.clone()
}

fn twice(verified: &Verified) -> Verified {
    copy(verified)
}

fn main() {
    let _ = twice;
}
