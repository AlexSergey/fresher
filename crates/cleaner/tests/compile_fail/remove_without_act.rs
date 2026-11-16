//! No removal can be reached from outside the cleaner: the only way to act is `Cleaner::execute` with a confirmed
//! plan, whose `Act` every removal takes.

use fresher_cleaner::remove::to_trash;

fn main() {
    let _ = to_trash;
}
