//! The words an installer's name may carry, as data: `installers.toml`, parsed and checked when the module is built.

use serde::Deserialize;

/// The table shipped with the module.
const SHIPPED: &str = include_str!("../installers.toml");

/// Why the words table was refused.
#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub enum WordsError {
    #[error("the words table is not valid TOML: {0}")]
    Format(String),
    #[error("the words table lists nothing")]
    Empty,
    #[error("`{word}` is not lowercase ASCII letters and digits")]
    BadWord { word: String },
    #[error("`{word}` is listed twice")]
    Duplicate { word: String },
}

/// The words an installer's name may carry after the app's name, checked.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Words {
    words: Vec<String>,
}

impl Words {
    pub(crate) fn shipped() -> Result<Self, WordsError> {
        Self::parse(SHIPPED)
    }

    /// Words read from `text`: at least one, each lowercase ASCII letters and digits, none twice.
    pub(crate) fn parse(text: &str) -> Result<Self, WordsError> {
        let table: RawTable = toml::from_str(text).map_err(|error| WordsError::Format(error.to_string()))?;
        if table.words.is_empty() {
            return Err(WordsError::Empty);
        }
        let mut words: Vec<String> = Vec::with_capacity(table.words.len());
        for word in table.words {
            let plain = !word.is_empty() && word.bytes().all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit());
            if !plain {
                return Err(WordsError::BadWord { word });
            }
            if words.contains(&word) {
                return Err(WordsError::Duplicate { word });
            }
            words.push(word);
        }
        Ok(Self { words })
    }

    /// Whether `word` is one of the words; the case is ignored.
    pub(crate) fn holds(&self, word: &str) -> bool {
        self.words.iter().any(|known| known.eq_ignore_ascii_case(word))
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawTable {
    #[serde(default)]
    words: Vec<String>,
}

#[cfg(test)]
mod tests {
    mod negative {
        use crate::installers::{Words, WordsError};

        #[test]
        fn text_that_is_not_toml_is_refused() {
            assert!(matches!(Words::parse("words = ["), Err(WordsError::Format(_))));
        }

        #[test]
        fn an_unknown_field_is_refused() {
            assert!(matches!(Words::parse("words = [\"mac\"]\nmore = 1"), Err(WordsError::Format(_))));
        }

        #[test]
        fn no_word_is_refused() {
            assert_eq!(Words::parse("words = []"), Err(WordsError::Empty));
            assert_eq!(Words::parse(""), Err(WordsError::Empty));
        }

        #[test]
        fn a_word_that_is_not_lowercase_ascii_letters_and_digits_is_refused() {
            for word in ["", "Mac", "set up", "x-64", "\u{e9}t\u{e9}"] {
                let text = format!("words = [\"{word}\"]");
                assert_eq!(Words::parse(&text), Err(WordsError::BadWord { word: word.into() }), "{word:?}");
            }
        }

        #[test]
        fn a_word_listed_twice_is_refused() {
            assert_eq!(Words::parse("words = [\"mac\", \"mac\"]"), Err(WordsError::Duplicate { word: "mac".into() }));
        }

        #[test]
        fn the_shipped_words_do_not_hold_backup() {
            let words = Words::shipped().expect("the shipped words are valid");

            assert!(!words.holds("backup"));
        }
    }

    mod positive {
        use crate::installers::Words;

        #[test]
        fn the_shipped_words_hold_installer_and_arm64() {
            let words = Words::shipped().expect("the shipped words are valid");

            assert!(words.holds("installer"));
            assert!(words.holds("arm64"));
            assert!(words.holds("ARM64"));
        }
    }
}
