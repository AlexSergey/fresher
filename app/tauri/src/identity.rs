//! What each build must run with: a development build never opens the release's data folder, and a build without
//! debug assertions (one that really removes) never runs as the development app nor with its page served by a
//! development server.

/// The suffix of the development identifier (`tauri.dev.conf.json`).
const DEVELOPMENT_SUFFIX: &str = ".dev";

/// Where the window's page comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Page {
    /// Embedded in the binary (`tauri build`, Tauri's `custom-protocol` feature on): the CSP holds.
    Embedded,
    /// Loaded from the `devUrl` (a plain Cargo build or `tauri dev`): whatever serves that port owns the page.
    DevServer,
}

impl Page {
    /// The page of this build: Tauri's dev mode loads it from the `devUrl`.
    pub(crate) fn of_this_build() -> Self {
        if tauri::is_dev() { Self::DevServer } else { Self::Embedded }
    }
}

/// Refuses a debug build whose identifier does not end in `.dev`, and a build without debug assertions whose identifier
/// does or whose page is not embedded. Such a build (`tauri dev --release`, `cargo build --release`, or debug
/// assertions turned off) has no dry run and no record, so its page must be the one built into it.
pub(crate) fn check(identifier: &str, debug_build: bool, page: Page) -> Result<(), String> {
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
    if !debug_build && page == Page::DevServer {
        return Err("a build without debug assertions really removes and must embed its page: \
                    build it with npm run build, never cargo build --release or tauri dev --release"
            .to_owned());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    mod negative {
        use super::super::{Page, check};

        #[test]
        fn a_debug_build_with_the_release_identifier_is_refused() {
            assert!(check("io.github.alexsergey.fresher", true, Page::DevServer).is_err());
            assert!(check("io.github.alexsergey.fresher", true, Page::Embedded).is_err());
        }

        #[test]
        fn a_release_build_with_the_development_identifier_is_refused() {
            assert!(check("io.github.alexsergey.fresher.dev", false, Page::Embedded).is_err());
        }

        #[test]
        fn a_release_build_whose_page_comes_from_a_development_server_is_refused() {
            assert!(check("io.github.alexsergey.fresher", false, Page::DevServer).is_err());
        }

        #[cfg(not(debug_assertions))]
        #[test]
        fn this_release_build_refuses_the_development_identifier() {
            assert!(check("io.github.alexsergey.fresher.dev", cfg!(debug_assertions), Page::Embedded).is_err());
        }

        #[cfg(not(debug_assertions))]
        #[test]
        fn this_plain_cargo_release_build_is_refused() {
            // `cargo test --release` builds without Tauri's `custom-protocol`, as `cargo build --release` does.
            assert_eq!(Page::of_this_build(), Page::DevServer);
            assert!(check("io.github.alexsergey.fresher", cfg!(debug_assertions), Page::of_this_build()).is_err());
        }

        #[cfg(debug_assertions)]
        #[test]
        fn this_debug_build_refuses_the_release_identifier() {
            assert!(check("io.github.alexsergey.fresher", cfg!(debug_assertions), Page::of_this_build()).is_err());
        }
    }

    mod positive {
        use super::super::{Page, check};

        #[test]
        fn the_development_identifier_is_accepted_in_a_debug_build() {
            assert!(check("io.github.alexsergey.fresher.dev", true, Page::DevServer).is_ok());
            assert!(check("io.github.alexsergey.fresher.dev", true, Page::Embedded).is_ok());
        }

        #[test]
        fn a_release_build_with_its_page_embedded_accepts_the_release_identifier() {
            assert!(check("io.github.alexsergey.fresher", false, Page::Embedded).is_ok());
        }

        #[cfg(debug_assertions)]
        #[test]
        fn this_debug_build_accepts_the_development_identifier() {
            assert!(check("io.github.alexsergey.fresher.dev", cfg!(debug_assertions), Page::of_this_build()).is_ok());
        }
    }
}
