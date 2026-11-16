//! The macOS adapters of Fresher: the only crate with unsafe code.
//!
//! It implements the core's OS ports for macOS, sets the process's IO policy, and gives the cleaner the calls that act
//! beneath a held folder ([`beneath`]). Every FFI call lives in the module
//! `ffi`, one unsafe operation per block, each with a `// SAFETY:` comment; everything else here is safe Rust. On any
//! other OS the crate is empty.

#![cfg(target_os = "macos")]

pub mod beneath;
mod bulk;
#[cfg(test)]
mod disk_image;
mod errors;
#[expect(unsafe_code, reason = "every FFI call of the app lives here, each block with its SAFETY comment")]
mod ffi;
#[cfg(any(test, feature = "fixtures"))]
pub mod fixtures;
pub mod io_policy;
mod journal;
mod open_files;
mod places;
mod process;
mod protection;
mod trash;
mod volume;

pub use bulk::MacFolderReader;
pub use journal::MacChangeJournal;
pub use open_files::MacOpenFiles;
pub use places::places_that_prompt;
pub use process::{current_user, peak_memory};
pub use protection::protection;
pub use trash::MacTrash;
pub use volume::MacVolumes;

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    fn sources(folder: &Path, found: &mut Vec<PathBuf>) {
        for entry in fs::read_dir(folder).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                sources(&path, found);
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                found.push(path);
            }
        }
    }

    mod negative {
        use std::fs;
        use std::path::Path;

        use super::sources;

        #[test]
        fn no_unsafe_outside_the_ffi_module() {
            let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
            let ffi = root.join("ffi");
            let forms = [
                concat!("unsafe", " {"),
                concat!("unsafe", " fn"),
                concat!("unsafe", " impl"),
                concat!("unsafe", " extern"),
                concat!("unsafe", "("),
            ];
            let mut files = Vec::new();
            sources(&root, &mut files);

            let offenders: Vec<_> = files
                .iter()
                .filter(|file| !file.starts_with(&ffi))
                .filter(|file| {
                    let text = fs::read_to_string(file).unwrap();
                    forms.iter().any(|form| text.contains(form))
                })
                .collect();

            assert!(files.len() > 3, "the sources were not found");
            assert!(offenders.is_empty(), "unsafe code outside src/ffi: {offenders:?}");
        }
    }
}
