//! A small glob of our own on one name segment: `*` matches any run of characters, `?` exactly one.

/// The characters a glob refuses: other globbing syntax and anything that is not part of one name.
const FORBIDDEN: [char; 7] = ['[', ']', '{', '}', '\\', '/', '\0'];

/// A glob on one file or folder name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Glob {
    pattern: Vec<char>,
}

impl Glob {
    /// A glob of `pattern`; `None` when it is empty or holds anything but name characters, `*` and `?`.
    pub(crate) fn new(pattern: &str) -> Option<Self> {
        if pattern.is_empty() || pattern == "." || pattern == ".." || pattern.contains(FORBIDDEN) {
            return None;
        }
        Some(Self { pattern: pattern.chars().collect() })
    }

    /// `true` when the whole of `name` matches, character by character.
    pub(crate) fn matches(&self, name: &str) -> bool {
        matches_chars(&self.pattern, &name.chars().collect::<Vec<_>>())
    }

    /// `true` when `name` matches with case ignored: the check of a `must-not-match`, so that a name a case-insensitive
    /// disk treats as the same is refused too.
    pub(crate) fn matches_ignoring_case(&self, name: &str) -> bool {
        let lower = |chars: &mut dyn Iterator<Item = char>| chars.flat_map(char::to_lowercase).collect::<Vec<_>>();
        matches_chars(&lower(&mut self.pattern.iter().copied()), &lower(&mut name.chars()))
    }
}

/// Matches with one backtracking point: the last `*` seen takes one more character whenever the rest fails.
fn matches_chars(pattern: &[char], name: &[char]) -> bool {
    let (mut at_pattern, mut at_name) = (0, 0);
    let mut last_star: Option<(usize, usize)> = None;
    while at_name < name.len() {
        match pattern.get(at_pattern) {
            Some('*') => {
                last_star = Some((at_pattern, at_name));
                at_pattern += 1;
            }
            Some(&wanted) if wanted == '?' || name.get(at_name) == Some(&wanted) => {
                at_pattern += 1;
                at_name += 1;
            }
            _ => match last_star {
                Some((star, taken)) => {
                    last_star = Some((star, taken + 1));
                    at_pattern = star + 1;
                    at_name = taken + 1;
                }
                None => return false,
            },
        }
    }
    pattern.get(at_pattern..).is_some_and(|rest| rest.iter().all(|&char| char == '*'))
}

#[cfg(test)]
mod tests {
    mod negative {
        use crate::glob::Glob;

        #[test]
        fn rejects_brackets_braces_and_paths() {
            for pattern in ["", ".", "..", "a[bc]", "{a,b}", "a\\*", "a/b", "a\0"] {
                assert_eq!(Glob::new(pattern), None, "{pattern:?}");
            }
        }

        #[test]
        fn does_not_match_a_look_alike() {
            let glob = Glob::new("com.example.*").unwrap();

            for name in ["com.example", "com.examples.app", "org.com.example.app", "Com.Example.app"] {
                assert!(!glob.matches(name), "{name}");
            }
            assert!(!Glob::new("a?c").unwrap().matches("ac"));
            assert!(!Glob::new("*.log").unwrap().matches("a.log.gz"));
        }
    }

    mod positive {
        use crate::glob::Glob;

        #[test]
        fn accepts_stars_and_question_marks() {
            let glob = Glob::new("com.*.cache-?").unwrap();

            assert_eq!(glob.pattern.len(), 13);
            assert!(glob.matches("com.app.cache-1"));
            assert!(glob.matches("com..cache-x"));
        }

        #[test]
        fn a_star_backtracks_and_may_match_nothing() {
            assert!(Glob::new("*.log").unwrap().matches("a.log.log"));
            assert!(Glob::new("a*b*c").unwrap().matches("abxbc"));
            assert!(Glob::new("Caches*").unwrap().matches("Caches"));
            assert!(Glob::new("**").unwrap().matches(""));
            assert!(Glob::new("Caches").unwrap().matches("Caches"));
        }

        #[test]
        fn ignoring_case_is_only_for_must_not_match() {
            let glob = Glob::new("com.apple.*").unwrap();

            assert!(glob.matches_ignoring_case("COM.Apple.Safari"));
            assert!(!glob.matches("COM.Apple.Safari"));
        }
    }

    mod property {
        use proptest::prelude::*;

        use crate::glob::Glob;

        proptest! {
            #[test]
            fn glob_star_matches_any_suffix(prefix in "[a-zA-Z0-9._-]{1,12}", suffix in "\\PC{0,16}") {
                let glob = Glob::new(&format!("{prefix}*")).unwrap();

                let name = format!("{prefix}{suffix}");

                prop_assert!(glob.matches(&name));
            }

            #[test]
            fn a_name_without_wildcards_matches_only_itself(name in "[a-z.]{1,8}", other in "[a-z.]{1,8}") {
                prop_assume!(name != "." && name != "..");
                let glob = Glob::new(&name).unwrap();

                prop_assert_eq!(glob.matches(&other), name == other);
            }
        }
    }
}
