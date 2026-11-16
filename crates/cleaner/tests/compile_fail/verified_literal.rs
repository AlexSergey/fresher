//! A checked item cannot be made by hand: only the guard makes one.

use fresher_cleaner::Verified;

fn forge() -> Verified {
    Verified {}
}

fn main() {
    let _ = forge;
}
