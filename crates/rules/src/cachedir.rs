//! `CACHEDIR.TAG`: a file a tool writes into a folder to say "this folder is a cache". It counts only when it starts
//! with the standard signature line.

use std::fs::OpenOptions;
use std::io::Read;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;

/// The name of the tag file.
pub(crate) const TAG: &str = "CACHEDIR.TAG";

/// The first bytes of a valid tag.
const SIGNATURE: &[u8] = b"Signature: 8a477f597d28d172789f06886806bc55";

/// `true` when `content` starts with the signature.
fn is_valid_tag(content: &[u8]) -> bool {
    content.starts_with(SIGNATURE)
}

/// `true` when `folder` holds a regular file `CACHEDIR.TAG` (not a link) that starts with the signature. Reads the
/// signature's length and no more; any error means "not a cache".
///
/// The file is opened once and checked through its handle: `O_NOFOLLOW` refuses a link, `O_NONBLOCK` keeps a FIFO or
/// a device from blocking the open, and the handle's own metadata must say "regular file". Nothing can be swapped in
/// between a check and the read.
pub(crate) fn holds_valid_tag(folder: &Path) -> bool {
    let opened = OpenOptions::new().read(true).custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK).open(folder.join(TAG));
    let Ok(file) = opened else { return false };
    if !file.metadata().is_ok_and(|metadata| metadata.is_file()) {
        return false;
    }
    let mut head = Vec::with_capacity(SIGNATURE.len());
    file.take(SIGNATURE.len() as u64).read_to_end(&mut head).is_ok() && is_valid_tag(&head)
}

#[cfg(test)]
pub(crate) mod sample {
    use fresher_testing::FixtureTree;

    /// A fixture with a folder `cache` whose `CACHEDIR.TAG` holds `content`.
    pub(crate) fn tagged(content: &[u8]) -> FixtureTree {
        let tree = FixtureTree::from_toml_str("[[dir]]\npath = \"cache\"\n").unwrap();
        std::fs::write(tree.root().join("cache/CACHEDIR.TAG"), content).unwrap();
        tree
    }
}

#[cfg(test)]
mod tests {

    mod negative {
        use std::process::Command;

        use crate::cachedir::holds_valid_tag;
        use crate::cachedir::sample::tagged;
        use fresher_testing::FixtureTree;

        #[test]
        fn a_fifo_named_like_the_tag_is_not_valid_and_does_not_block() {
            let tree = FixtureTree::from_toml_str("[[dir]]\npath = \"cache\"\n").unwrap();
            let fifo = tree.root().join("cache/CACHEDIR.TAG");
            assert!(Command::new("mkfifo").arg(&fifo).status().unwrap().success());

            assert!(!holds_valid_tag(&tree.root().join("cache")));
        }

        #[test]
        fn a_tag_linked_to_a_valid_tag_is_not_valid() {
            let tree = tagged(b"Signature: 8a477f597d28d172789f06886806bc55");
            std::fs::create_dir(tree.root().join("linked")).unwrap();
            std::os::unix::fs::symlink("../cache/CACHEDIR.TAG", tree.root().join("linked/CACHEDIR.TAG")).unwrap();

            assert!(holds_valid_tag(&tree.root().join("cache")));
            assert!(!holds_valid_tag(&tree.root().join("linked")));
        }

        #[test]
        fn a_tag_without_the_signature_is_not_valid() {
            for content in [&b""[..], b"Signature: 0000", b"signature: 8a477f597d28d172789f06886806bc55"] {
                let tree = tagged(content);

                assert!(!holds_valid_tag(&tree.root().join("cache")), "{content:?}");
            }
        }

        #[test]
        fn a_missing_tag_a_tag_folder_or_a_linked_tag_is_not_valid() {
            let tree = FixtureTree::from_toml_str(
                "[[dir]]\npath = \"none\"\n[[dir]]\npath = \"folder/CACHEDIR.TAG\"\n\
                 [[symlink]]\npath = \"linked/CACHEDIR.TAG\"\ntarget = \"../real/CACHEDIR.TAG\"\n",
            )
            .unwrap();
            std::fs::create_dir_all(tree.root().join("real")).unwrap();
            std::fs::write(tree.root().join("real/CACHEDIR.TAG"), b"Signature: 8a477f597d28d172789f06886806bc55")
                .unwrap();

            for folder in ["none", "folder", "linked"] {
                assert!(!holds_valid_tag(&tree.root().join(folder)), "{folder}");
            }
            assert!(holds_valid_tag(&tree.root().join("real")));
        }
    }

    mod positive {
        use crate::cachedir::holds_valid_tag;
        use crate::cachedir::sample::tagged;

        #[test]
        fn a_tag_with_the_signature_is_valid() {
            let tree = tagged(b"Signature: 8a477f597d28d172789f06886806bc55\n# This file is a cache directory tag.\n");

            assert!(holds_valid_tag(&tree.root().join("cache")));
        }
    }
}
