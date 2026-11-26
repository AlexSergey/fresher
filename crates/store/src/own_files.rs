//! The names of the files the store writes atomically: the targets of its temporary files.

use crate::CachePolicy;
use crate::scan_cache::current_file_name;
use crate::settings_file::FILE_NAME as SETTINGS;

/// The names of the files the store writes atomically (`settings.json` and the scan cache's current files under
/// `policy`). A temporary file of one of them is `.<name>.<random>.tmp` (`atomic.rs`); the journal is appended to and
/// the cache's previous files are renames, so none of those has one.
pub fn atomically_written_files(policy: CachePolicy) -> Vec<String> {
    let slots = (1..=policy.scopes.get()).map(current_file_name);
    std::iter::once(SETTINGS.to_owned()).chain(slots).collect()
}

#[cfg(test)]
mod tests {
    mod positive {
        use crate::{CachePolicy, atomically_written_files};

        #[test]
        fn names_the_settings_and_each_current_cache_file() {
            assert_eq!(atomically_written_files(CachePolicy::DEFAULT), ["settings.json", "scan-1.bin", "scan-2.bin"]);
        }
    }
}
