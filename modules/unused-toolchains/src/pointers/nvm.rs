//! nvm: version folder names, and references (an alias file's or a pin's first line) resolved to the installed
//! versions they keep.

use std::collections::{BTreeMap, BTreeSet};

use super::{Resolved, numbers, without_bom};

/// The longest chain of aliases followed.
const MAX_HOPS: usize = 8;

/// The parts of a folder named `v<major>.<minor>.<patch>`; any other name is off shape.
pub(crate) fn folder(name: &str) -> Option<[u32; 3]> {
    let [major, minor, patch] = numbers(name.strip_prefix('v')?)?[..] else {
        return None;
    };
    Some([major, minor, patch])
}

/// What the reference in `text` keeps among `installed`, following `aliases` (keyed by the alias file's path under
/// `alias/`, such as `default` or `lts/iron`).
pub(crate) fn resolve(text: &str, aliases: &BTreeMap<String, String>, installed: &[[u32; 3]]) -> Resolved {
    let mut current = first_line(text);
    for _ in 0..=MAX_HOPS {
        if current == "system" {
            return Resolved::Nothing;
        }
        if current == "node" || current == "stable" {
            let newest = installed.iter().enumerate().max_by_key(|(_, version)| **version).map(|(index, _)| index);
            return Resolved::Keeps(newest.into_iter().collect());
        }
        if let Some(prefix) = prefix(current) {
            return Resolved::Keeps(
                installed
                    .iter()
                    .enumerate()
                    .filter(|(_, version)| version.starts_with(&prefix))
                    .map(|(index, _)| index)
                    .collect::<BTreeSet<_>>(),
            );
        }
        match aliases.get(current) {
            Some(target) => current = first_line(target),
            None => return Resolved::Unknown,
        }
    }
    Resolved::Unknown
}

fn first_line(text: &str) -> &str {
    without_bom(text).lines().next().unwrap_or_default().trim()
}

/// The one to three numbers of `v?X`, `v?X.Y` or `v?X.Y.Z`.
fn prefix(reference: &str) -> Option<Vec<u32>> {
    let numbers = numbers(reference.strip_prefix('v').unwrap_or(reference))?;
    (1..=3).contains(&numbers.len()).then_some(numbers)
}

#[cfg(test)]
mod tests {
    use super::*;

    const INSTALLED: [[u32; 3]; 4] = [[18, 19, 0], [20, 1, 0], [18, 20, 4], [16, 0, 0]];

    fn aliases(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
        pairs.iter().map(|(name, target)| ((*name).to_owned(), (*target).to_owned())).collect()
    }

    fn keeps(indices: &[usize]) -> Resolved {
        Resolved::Keeps(indices.iter().copied().collect())
    }

    mod negative {
        use super::*;

        #[test]
        fn a_name_off_v_x_y_z_is_off_shape() {
            for name in
                ["18.1.0", "v18", "v18.1.0-rc.1", "system", "v18.1", "v1.2.3.4", "v-1.2.3", ".DS_Store", "v1..3"]
            {
                assert_eq!(folder(name), None, "{name}");
            }
        }

        #[test]
        fn an_unknown_word_is_unknown() {
            assert_eq!(resolve("iron", &BTreeMap::new(), &INSTALLED), Resolved::Unknown);
            assert_eq!(resolve("", &BTreeMap::new(), &INSTALLED), Resolved::Unknown);
            assert_eq!(resolve("18.1.0-rc.1", &BTreeMap::new(), &INSTALLED), Resolved::Unknown);
        }

        #[test]
        fn an_lts_name_without_its_alias_file_is_unknown() {
            let aliases = aliases(&[("lts/hydrogen", "v18.19.0")]);

            assert_eq!(resolve("lts/iron", &aliases, &INSTALLED), Resolved::Unknown);
            assert_eq!(resolve("lts/*", &aliases, &INSTALLED), Resolved::Unknown);
        }

        #[test]
        fn a_loop_or_a_long_chain_is_unknown() {
            let looped = aliases(&[("a", "b"), ("b", "a")]);
            let chain: Vec<(String, String)> = (0..10).map(|i| (format!("a{i}"), format!("a{}", i + 1))).collect();
            let long: BTreeMap<String, String> = chain.into_iter().collect();

            assert_eq!(resolve("a", &looped, &INSTALLED), Resolved::Unknown);
            assert_eq!(resolve("a0", &long, &INSTALLED), Resolved::Unknown);
        }

        #[test]
        fn system_is_nothing() {
            assert_eq!(resolve("system\n", &BTreeMap::new(), &INSTALLED), Resolved::Nothing);
        }

        #[test]
        fn a_leading_bom_does_not_hide_the_reference() {
            assert_eq!(resolve("\u{feff}system", &BTreeMap::new(), &INSTALLED), Resolved::Nothing);
        }

        #[test]
        fn lts_and_latest_are_unknown_without_their_alias() {
            assert_eq!(resolve("latest", &BTreeMap::new(), &INSTALLED), Resolved::Unknown);
            assert_eq!(resolve("lts", &BTreeMap::new(), &INSTALLED), Resolved::Unknown);
        }

        #[test]
        fn a_version_that_is_not_installed_keeps_nothing() {
            assert_eq!(resolve("v22", &BTreeMap::new(), &INSTALLED), keeps(&[]));
        }
    }

    mod positive {
        use proptest::prelude::*;

        use super::*;

        #[test]
        fn a_leading_bom_and_crlf_are_read_through() {
            assert_eq!(resolve("\u{feff}v20.1.0\r\n", &BTreeMap::new(), &INSTALLED), keeps(&[1]));
            assert_eq!(resolve("  18  \r\n", &BTreeMap::new(), &INSTALLED), keeps(&[0, 2]));
        }

        #[test]
        fn a_folder_is_its_three_numbers() {
            assert_eq!(folder("v18.19.0"), Some([18, 19, 0]));
        }

        #[test]
        fn node_and_stable_keep_the_newest() {
            assert_eq!(resolve("node", &BTreeMap::new(), &INSTALLED), keeps(&[1]));
            assert_eq!(resolve("stable", &BTreeMap::new(), &INSTALLED), keeps(&[1]));
        }

        #[test]
        fn a_major_keeps_every_installed_version_of_it() {
            assert_eq!(resolve("18", &BTreeMap::new(), &INSTALLED), keeps(&[0, 2]));
            assert_eq!(resolve("v18.19", &BTreeMap::new(), &INSTALLED), keeps(&[0]));
        }

        #[test]
        fn an_exact_version_keeps_only_itself() {
            assert_eq!(resolve("v20.1.0\n", &BTreeMap::new(), &INSTALLED), keeps(&[1]));
        }

        #[test]
        fn lts_star_resolves_through_its_alias_files() {
            let aliases = aliases(&[("lts/*", "lts/hydrogen"), ("lts/hydrogen", "v18.19.0")]);

            assert_eq!(resolve("lts/*", &aliases, &INSTALLED), keeps(&[0]));
        }

        #[test]
        fn a_user_alias_resolves_to_its_version() {
            let aliases = aliases(&[("work", "16")]);

            assert_eq!(resolve("work", &aliases, &INSTALLED), keeps(&[3]));
        }

        #[test]
        fn a_chain_of_eight_aliases_resolves() {
            let pairs: Vec<(String, String)> = (0..7).map(|i| (format!("a{i}"), format!("a{}", i + 1))).collect();
            let mut chain: BTreeMap<String, String> = pairs.into_iter().collect();
            chain.insert("a7".to_owned(), "system".to_owned());

            assert_eq!(resolve("a0", &chain, &INSTALLED), Resolved::Nothing);
        }

        proptest! {
            #[test]
            fn resolve_never_keeps_an_index_outside_installed(
                text in "[a-z0-9./*v-]{0,12}",
                installed in proptest::collection::vec([0u32..30, 0u32..30, 0u32..30], 0..6),
            ) {
                let aliases = aliases(&[("default", "node"), ("lts/*", "v18"), ("a", "a")]);

                if let Resolved::Keeps(indices) = resolve(&text, &aliases, &installed) {
                    prop_assert!(indices.iter().all(|index| *index < installed.len()));
                }
            }
        }
    }
}
