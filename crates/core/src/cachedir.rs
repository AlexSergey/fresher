//! `CACHEDIR.TAG`: a file a tool writes into a folder to say "this folder is a cache". It counts only when it starts
//! with the standard signature line. The walk reads the file; this module only judges the bytes read.

/// The name of the tag file.
pub const CACHEDIR_TAG: &str = "CACHEDIR.TAG";

/// The first bytes of a valid tag; the walk reads this many bytes and no more.
pub const CACHEDIR_SIGNATURE: &[u8] = b"Signature: 8a477f597d28d172789f06886806bc55";

/// `true` when `head`, the first bytes of a `CACHEDIR.TAG`, starts with the signature.
pub fn is_valid_cachedir_tag(head: &[u8]) -> bool {
    head.starts_with(CACHEDIR_SIGNATURE)
}

#[cfg(test)]
mod tests {
    mod negative {
        use crate::is_valid_cachedir_tag;

        #[test]
        fn an_empty_tag_is_not_valid() {
            assert!(!is_valid_cachedir_tag(b""));
        }

        #[test]
        fn a_tag_without_the_signature_is_not_valid() {
            for head in [
                &b"Signature: 0000"[..],
                b"signature: 8a477f597d28d172789f06886806bc55",
                b"Signature: 8a477f597d28d172789f06886806bc5",
                b" Signature: 8a477f597d28d172789f06886806bc55",
            ] {
                assert!(!is_valid_cachedir_tag(head), "{head:?}");
            }
        }
    }

    mod positive {
        use crate::{CACHEDIR_SIGNATURE, is_valid_cachedir_tag};

        #[test]
        fn a_tag_with_the_signature_is_valid() {
            assert!(is_valid_cachedir_tag(CACHEDIR_SIGNATURE));
            assert!(is_valid_cachedir_tag(
                b"Signature: 8a477f597d28d172789f06886806bc55\n# This file is a cache directory tag.\n"
            ));
        }
    }
}
