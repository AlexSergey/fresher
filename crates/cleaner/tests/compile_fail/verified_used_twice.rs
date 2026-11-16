//! A checked item is consumed by the act it was checked for: a second act cannot take it.

use fresher_cleaner::Verified;

fn act(_verified: Verified) {}

fn twice(verified: Verified) {
    act(verified);
    act(verified);
}

fn main() {
    let _ = twice;
}
