//! The identifier a development build must run with, so it never opens the release's data folder.

/// The suffix of the development identifier (`tauri.dev.conf.json`).
const DEVELOPMENT_SUFFIX: &str = ".dev";

/// Refuses a debug build that runs with an identifier not ending in `.dev`; a release build is never refused.
pub(crate) fn check(identifier: &str, debug_build: bool) -> Result<(), String> {
    if debug_build && !identifier.ends_with(DEVELOPMENT_SUFFIX) {
        return Err(format!(
            "a development build must run with tauri.dev.conf.json: npm run dev (identifier {identifier})"
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::check;

    mod negative {
        use super::check;

        #[test]
        fn a_debug_build_with_the_release_identifier_is_refused() {
            assert!(check("io.github.alexsergey.fresher", true).is_err());
        }
    }

    mod positive {
        use super::check;

        #[test]
        fn the_development_identifier_is_accepted() {
            assert!(check("io.github.alexsergey.fresher.dev", true).is_ok());
        }

        #[test]
        fn a_release_build_accepts_the_release_identifier() {
            assert!(check("io.github.alexsergey.fresher", false).is_ok());
        }
    }
}
