//! The envelope of every file the app writes: `{ "format": …, "version": …, <body fields> }`.

use std::cmp::Ordering;
use std::io;

use fresher_core::{FormatVersion, StoreError, VersionPolicy};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// A file read through its envelope: its body, the fields next to `format` and `version`, as one JSON object.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Versioned {
    /// The version this build writes.
    Current(Value),
    /// An older version of a file that is migrated: the body still has to go through the migrations.
    Older { version: FormatVersion, body: Value },
    /// An older version of a file that is rebuilt: its body is not kept.
    Discarded { version: FormatVersion },
}

#[derive(Deserialize)]
struct EnvelopeIn {
    format: String,
    version: FormatVersion,
    #[serde(flatten)]
    body: Map<String, Value>,
}

#[derive(Serialize)]
struct EnvelopeOut<'a, B> {
    format: &'a str,
    version: FormatVersion,
    #[serde(flatten)]
    body: &'a B,
}

/// Reads a file of `format` that this build writes at version `supported`.
///
/// Not JSON, another format or a version below 1 is `Broken`; a version above `supported` is `NewerVersion`.
pub(crate) fn read_versioned(
    bytes: &[u8],
    format: &str,
    supported: FormatVersion,
    policy: VersionPolicy,
) -> Result<Versioned, StoreError> {
    let envelope: EnvelopeIn = serde_json::from_slice(bytes).map_err(|_| StoreError::Broken)?;
    if envelope.format != format || envelope.version.get() == 0 {
        return Err(StoreError::Broken);
    }
    let version = envelope.version;
    match (version.cmp(&supported), policy) {
        (Ordering::Greater, _) => Err(StoreError::NewerVersion { found: version, supported }),
        (Ordering::Equal, _) => Ok(Versioned::Current(Value::Object(envelope.body))),
        (Ordering::Less, VersionPolicy::Migrate) => {
            Ok(Versioned::Older { version, body: Value::Object(envelope.body) })
        }
        (Ordering::Less, VersionPolicy::Discard) => Ok(Versioned::Discarded { version }),
    }
}

/// The bytes of a file of `format` at `version`: pretty JSON, `format` and `version` first, ending with a newline.
pub(crate) fn write_versioned<B: Serialize>(
    format: &str,
    version: FormatVersion,
    body: &B,
) -> Result<Vec<u8>, StoreError> {
    let mut bytes = serde_json::to_vec_pretty(&EnvelopeOut { format, version, body })
        .map_err(|error| StoreError::Io { kind: io::Error::from(error).kind() })?;
    bytes.push(b'\n');
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use fresher_core::FormatVersion;

    const TWO: FormatVersion = FormatVersion::new(2);

    mod negative {
        use fresher_core::{FormatVersion, StoreError, VersionPolicy};
        use serde_json::json;

        use super::TWO;
        use crate::versioned::{read_versioned, write_versioned};

        fn read(bytes: &[u8]) -> Result<crate::versioned::Versioned, StoreError> {
            read_versioned(bytes, "settings", TWO, VersionPolicy::Migrate)
        }

        #[test]
        fn a_newer_version_is_an_error() {
            let bytes = write_versioned("settings", FormatVersion::new(3), &json!({ "values": {} })).unwrap();

            assert_eq!(read(&bytes), Err(StoreError::NewerVersion { found: FormatVersion::new(3), supported: TWO }));
        }

        #[test]
        fn another_format_is_broken() {
            let bytes = write_versioned("journal", TWO, &json!({ "values": {} })).unwrap();

            assert_eq!(read(&bytes), Err(StoreError::Broken));
        }

        #[test]
        fn a_file_without_an_envelope_is_broken() {
            for bytes in [
                &b""[..],
                b"not json",
                b"[]",
                br#"{ "values": {} }"#,
                br#"{ "format": "settings", "values": {} }"#,
                br#"{ "format": "settings", "version": "2" }"#,
                br#"{ "format": "settings", "version": -1 }"#,
                br#"{ "format": "settings", "version": 0 }"#,
            ] {
                assert_eq!(read(bytes), Err(StoreError::Broken), "{}", String::from_utf8_lossy(bytes));
            }
        }

        #[test]
        fn a_body_that_is_not_an_object_cannot_be_written() {
            let result = write_versioned("settings", TWO, &json!([1, 2]));

            assert!(matches!(result, Err(StoreError::Io { .. })), "{result:?}");
        }
    }

    mod positive {
        use fresher_core::{FormatVersion, VersionPolicy};
        use serde_json::json;

        use super::TWO;
        use crate::versioned::{Versioned, read_versioned, write_versioned};

        #[test]
        fn reads_the_current_version_with_its_body() {
            let body = json!({ "values": { "demo.enabled": true } });
            let bytes = write_versioned("settings", TWO, &body).unwrap();

            assert_eq!(read_versioned(&bytes, "settings", TWO, VersionPolicy::Migrate), Ok(Versioned::Current(body)));
        }

        #[test]
        fn an_older_version_is_reported_for_migration() {
            let body = json!({ "values": {} });
            let bytes = write_versioned("settings", FormatVersion::new(1), &body).unwrap();

            assert_eq!(
                read_versioned(&bytes, "settings", TWO, VersionPolicy::Migrate),
                Ok(Versioned::Older { version: FormatVersion::new(1), body })
            );
        }

        #[test]
        fn an_older_version_of_a_rebuilt_file_is_discarded() {
            let bytes = write_versioned("cache", FormatVersion::new(1), &json!({ "entries": [] })).unwrap();

            assert_eq!(
                read_versioned(&bytes, "cache", TWO, VersionPolicy::Discard),
                Ok(Versioned::Discarded { version: FormatVersion::new(1) })
            );
        }

        #[test]
        fn writes_the_envelope_first() {
            let bytes = write_versioned("settings", TWO, &json!({ "values": {} })).unwrap();

            assert_eq!(
                String::from_utf8(bytes).unwrap(),
                "{\n  \"format\": \"settings\",\n  \"version\": 2,\n  \"values\": {}\n}\n"
            );
        }
    }
}
