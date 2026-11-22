//! The system's privacy protection: which places it guards, and how to tell, without a prompt, what an open of one
//! answers. The tables are the design's "Probe results" (plan 006), from documentation and unverified until the
//! user's run of plan 006/01; they live in `places.rs` and `check.rs` only. Its own types: the mapping onto the core's
//! capabilities is `fresher-system`'s.

use std::path::{Path, PathBuf};

mod check;
mod panes;
mod places;
mod volumes;

pub use check::{Probe, classify, fda_candidates, full_disk_access, try_inside, try_place};
pub use panes::{OPEN, Pane, open_args, open_folder_args, pane_address, pane_of, reveal_args};
pub use places::{Guard, guarded_places, places_that_prompt};
pub use volumes::volume_guard;

/// `names` joined under `home`, one name per component. Pure: it builds a path and opens nothing.
fn under(home: &Path, names: &[&str]) -> PathBuf {
    names.iter().fold(home.to_path_buf(), |path, name| path.join(name))
}
