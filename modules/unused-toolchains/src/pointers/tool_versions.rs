//! The `.tool-versions` pin file, shared by several managers.

use super::without_bom;

/// One version a `.tool-versions` line names for a tool.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ToolVersion<'a> {
    /// A version or an alias, to be resolved by the manager.
    Version(&'a str),
    /// `system`, `ref:…` or `path:…`: no version of the manager.
    Nothing,
    /// A line of the tool that names no version: it cannot be read with certainty.
    Unknown,
}

/// The references of the lines whose tool is one of `keys`; comments and blank lines are skipped.
pub(crate) fn references<'a>(text: &'a str, keys: &[&str]) -> Vec<ToolVersion<'a>> {
    let mut found = Vec::new();
    for line in without_bom(text).lines() {
        let line = line.split('#').next().unwrap_or_default();
        let mut words = line.split_whitespace();
        if !words.next().is_some_and(|tool| keys.contains(&tool)) {
            continue;
        }
        let before = found.len();
        found.extend(words.map(|word| {
            if word == "system" || word.starts_with("ref:") || word.starts_with("path:") {
                ToolVersion::Nothing
            } else {
                ToolVersion::Version(word)
            }
        }));
        if found.len() == before {
            found.push(ToolVersion::Unknown);
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    mod negative {
        use super::*;

        #[test]
        fn another_tool_s_line_gives_nothing() {
            assert_eq!(references("python 3.12.1\nruby 3.3.0\n", &["nodejs"]).len(), 0);
        }

        #[test]
        fn system_ref_and_path_are_nothing() {
            let found = references("nodejs system ref:main path:/opt/node\n", &["nodejs"]);

            assert_eq!(found, [ToolVersion::Nothing; 3]);
        }

        #[test]
        fn comments_and_blank_lines_are_skipped() {
            let found = references("\n# nodejs 18.0.0\n   \nnodejs 20.1.0 # the current one\n", &["nodejs"]);

            assert_eq!(found, [ToolVersion::Version("20.1.0")]);
        }

        #[test]
        fn a_line_of_the_tool_without_a_version_is_unknown() {
            assert_eq!(references("nodejs\n", &["nodejs"]), [ToolVersion::Unknown]);
        }

        #[test]
        fn latest_and_lts_are_versions_for_the_manager_to_refuse() {
            let found = references("nodejs latest\nnodejs lts\n", &["nodejs"]);

            assert_eq!(found, [ToolVersion::Version("latest"), ToolVersion::Version("lts")]);
        }

        #[test]
        fn a_key_only_matches_the_whole_tool_name() {
            assert_eq!(references("nodejs-lts 20.1.0\n", &["nodejs"]).len(), 0);
        }
    }

    mod positive {
        use super::*;

        #[test]
        fn every_version_of_a_key_s_line_is_a_reference() {
            let found = references("nodejs 20.1.0 18.19.0\n", &["nodejs"]);

            assert_eq!(found, [ToolVersion::Version("20.1.0"), ToolVersion::Version("18.19.0")]);
        }

        #[test]
        fn a_leading_bom_crlf_tabs_and_trailing_blanks_are_read_through() {
            let found = references("\u{feff}nodejs\t20.1.0 \r\nrust 1.80.0  \r\n", &["nodejs", "rust"]);

            assert_eq!(found, [ToolVersion::Version("20.1.0"), ToolVersion::Version("1.80.0")]);
        }

        #[test]
        fn node_and_nodejs_both_count_for_nvm() {
            let found = references("node 16.0.0\nnodejs 20.1.0\nrust 1.80.0\n", &["nodejs", "node"]);

            assert_eq!(found, [ToolVersion::Version("16.0.0"), ToolVersion::Version("20.1.0")]);
        }
    }
}
