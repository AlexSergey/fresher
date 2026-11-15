//! Migrations of a file's body, one version step at a time.

use fresher_core::{FormatVersion, StoreError};
use serde_json::Value;

/// Takes a file's body one version forward. Fails with `Broken` when the old body is not what it expects.
pub(crate) type Migration = fn(Value) -> Result<Value, StoreError>;

/// Brings `body` from version `from` to version `to`: `migrations[i]` takes version `i + 1` to `i + 2`, and every step
/// between the two versions is applied in order. A chain that does not cover the steps is `Broken`.
pub(crate) fn migrate(
    body: Value,
    from: FormatVersion,
    to: FormatVersion,
    migrations: &[Migration],
) -> Result<Value, StoreError> {
    let index = |version: FormatVersion| usize::try_from(version.get().checked_sub(1)?).ok();
    let steps =
        index(from).zip(index(to)).and_then(|(start, end)| migrations.get(start..end)).ok_or(StoreError::Broken)?;
    steps.iter().try_fold(body, |body, step| step(body))
}

/// Migrations for tests: a two-step chain from version 1 to 3, and a step that always fails.
#[cfg(test)]
pub(crate) mod fixtures {
    use fresher_core::StoreError;
    use serde_json::{Value, json};

    use super::Migration;

    /// Version 1 → 2: the flat `enabled` becomes `demo.enabled`.
    fn rename_enabled(mut body: Value) -> Result<Value, StoreError> {
        let values = body["values"].as_object_mut().ok_or(StoreError::Broken)?;
        let enabled = values.remove("enabled").ok_or(StoreError::Broken)?;
        values.insert("demo.enabled".into(), enabled);
        Ok(body)
    }

    /// Version 2 → 3: a new setting with its value.
    #[expect(clippy::unnecessary_wraps, reason = "a step of the chain has the signature of a migration")]
    fn add_min_age(mut body: Value) -> Result<Value, StoreError> {
        body["values"]["demo.min-age"] = json!(30);
        Ok(body)
    }

    fn fails(_body: Value) -> Result<Value, StoreError> {
        Err(StoreError::Broken)
    }

    pub(crate) const TWO_STEPS: &[Migration] = &[rename_enabled, add_min_age];
    pub(crate) const FAILING: &[Migration] = &[fails];
}

#[cfg(test)]
mod tests {
    mod negative {
        use fresher_core::{FormatVersion, StoreError};
        use serde_json::json;

        use crate::migrate::fixtures::TWO_STEPS as CHAIN;
        use crate::migrate::migrate;

        #[test]
        fn a_failing_step_stops_the_chain() {
            let body = json!({ "values": { "other": true } });

            assert_eq!(migrate(body, FormatVersion::new(1), FormatVersion::new(3), CHAIN), Err(StoreError::Broken));
        }

        #[test]
        fn a_chain_that_does_not_reach_the_version_is_broken() {
            let body = json!({ "values": {} });

            for (from, to) in [(0, 3), (1, 4), (3, 2)] {
                let result = migrate(body.clone(), FormatVersion::new(from), FormatVersion::new(to), CHAIN);
                assert_eq!(result, Err(StoreError::Broken), "{from} → {to}");
            }
        }
    }

    mod positive {
        use fresher_core::FormatVersion;
        use serde_json::json;

        use crate::migrate::fixtures::TWO_STEPS as CHAIN;
        use crate::migrate::migrate;

        #[test]
        fn migrates_an_old_version_step_by_step() {
            let body = json!({ "values": { "enabled": false } });

            let migrated = migrate(body, FormatVersion::new(1), FormatVersion::new(3), CHAIN).unwrap();

            assert_eq!(migrated, json!({ "values": { "demo.enabled": false, "demo.min-age": 30 } }));
        }

        #[test]
        fn starts_from_the_version_of_the_file() {
            let body = json!({ "values": {} });

            let migrated = migrate(body, FormatVersion::new(2), FormatVersion::new(3), CHAIN).unwrap();

            assert_eq!(migrated, json!({ "values": { "demo.min-age": 30 } }));
        }

        #[test]
        fn the_current_version_needs_no_step() {
            let body = json!({ "values": { "enabled": true } });

            assert_eq!(migrate(body.clone(), FormatVersion::new(3), FormatVersion::new(3), CHAIN), Ok(body));
        }
    }
}
