//! A `Text` in English, from the engine's dictionary.

use fresher_core::{Dictionary, Text, TextArg};

use super::clock;

/// The template of the text's key with every `{name}` replaced by its argument; `None` for a key the dictionary
/// lacks. A placeholder the text does not set stays as written.
pub(crate) fn english(dictionary: &Dictionary, text: &Text) -> Option<String> {
    let mut written = dictionary.get(text.key().as_str())?.to_owned();
    for (name, arg) in text.args() {
        let value = match arg {
            TextArg::Text(value) => value.clone(),
            TextArg::Count(count) => count.to_string(),
            TextArg::Size(size) => format!("{} bytes", size.bytes()),
            TextArg::Date(date) => clock::utc(clock::at_secs(date.secs())).date(),
            TextArg::Path(path) => path.display().to_string(),
        };
        written = written.replace(&format!("{{{name}}}"), &value);
    }
    Some(written)
}

#[cfg(test)]
mod tests {
    fn dictionary(json: &str) -> fresher_core::Dictionary {
        fresher_core::Dictionary::parse("demo", json).unwrap()
    }

    fn key(key: &str) -> fresher_core::TextKey {
        fresher_core::TextKey::new(key).unwrap()
    }

    mod negative {
        use fresher_core::{Text, TextArg};

        use super::{dictionary, key};
        use crate::dev_runs::english::english;

        #[test]
        fn a_key_the_dictionary_lacks_has_no_english() {
            let dictionary = dictionary(r#"{"demo.found": "Found"}"#);

            assert_eq!(english(&dictionary, &Text::new(key("demo.other"))), None);
        }

        #[test]
        fn an_unset_placeholder_stays_as_written() {
            let dictionary = dictionary(r#"{"demo.found": "{count} in {path}"}"#);
            let text = Text::new(key("demo.found")).with("count", TextArg::Count(3));

            assert_eq!(english(&dictionary, &text).as_deref(), Some("3 in {path}"));
        }
    }

    mod positive {
        use std::path::PathBuf;

        use fresher_core::{ByteSize, Text, TextArg, Timestamp};

        use super::{dictionary, key};
        use crate::dev_runs::english::english;

        #[test]
        fn every_kind_of_argument_is_written() {
            let dictionary = dictionary(r#"{"demo.found": "{app}: {count} items, {size}, since {date}, at {path}"}"#);
            let text = Text::new(key("demo.found"))
                .with("app", TextArg::Text("Editor".into()))
                .with("count", TextArg::Count(2))
                .with("size", TextArg::Size(ByteSize::new(1234)))
                .with("date", TextArg::Date(Timestamp::from_secs(1_763_633_071)))
                .with("path", TextArg::Path(PathBuf::from("/Users/x/old")));

            assert_eq!(
                english(&dictionary, &text).as_deref(),
                Some("Editor: 2 items, 1234 bytes, since 2025-11-20, at /Users/x/old")
            );
        }
    }
}
