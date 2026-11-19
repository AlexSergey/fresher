//! The identifier each build must run with: a development build never opens the release's data folder, and a build
//! without debug assertions (one that really removes) never runs as the development app.

/// The suffix of the development identifier (`tauri.dev.conf.json`).
const DEVELOPMENT_SUFFIX: &str = ".dev";

/// Refuses a debug build whose identifier does not end in `.dev`, and a build without debug assertions whose identifier
/// does: such a build (`tauri dev --release`, or debug assertions turned off) has no dry run and no record.
pub(crate) fn check(identifier: &str, debug_build: bool) -> Result<(), String> {
    let development = identifier.ends_with(DEVELOPMENT_SUFFIX);
    if debug_build && !development {
        return Err(format!(
            "a development build must run with tauri.dev.conf.json: npm run dev (identifier {identifier})"
        ));
    }
    if !debug_build && development {
        return Err(format!(
            "a build without debug assertions really removes and must not run as the development app: \
             use npm run dev, never --release or debug assertions off (identifier {identifier})"
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

        #[test]
        fn a_release_build_with_the_development_identifier_is_refused() {
            assert!(check("io.github.alexsergey.fresher.dev", false).is_err());
        }

        #[cfg(not(debug_assertions))]
        #[test]
        fn this_release_build_refuses_the_development_identifier() {
            assert!(check("io.github.alexsergey.fresher.dev", cfg!(debug_assertions)).is_err());
        }

        #[cfg(debug_assertions)]
        #[test]
        fn this_debug_build_refuses_the_release_identifier() {
            assert!(check("io.github.alexsergey.fresher", cfg!(debug_assertions)).is_err());
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
