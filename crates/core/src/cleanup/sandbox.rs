//! The sandbox marker: in a development build or a test, the cleaner removes only below a folder that holds it.

/// The marker's file name, in a scan root or a folder between the root and the item.
pub const SANDBOX_MARKER: &str = ".fresher-sandbox";

/// The first line of the marker: a file that does not start with it is not a marker.
pub const SANDBOX_SIGNATURE: &str =
    "fresher-sandbox 1: development builds and tests may remove files below this folder";

#[cfg(test)]
mod tests {
    mod positive {
        use crate::cleanup::{SANDBOX_MARKER, SANDBOX_SIGNATURE};

        #[test]
        fn the_marker_is_a_hidden_file_with_a_one_line_signature() {
            assert!(SANDBOX_MARKER.starts_with('.') && !SANDBOX_MARKER.contains('/'));
            assert!(!SANDBOX_SIGNATURE.contains('\n'));
        }
    }
}
