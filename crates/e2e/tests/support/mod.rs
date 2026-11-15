//! What the end-to-end tests share: the fixtures folder and the golden files.

use std::path::PathBuf;

pub(crate) fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

/// Compares `report`, as pretty JSON, with `fixtures/golden/<name>.json`; `UPDATE_GOLDEN=1` rewrites the file instead.
pub(crate) fn check_golden(name: &str, report: &impl serde::Serialize) {
    let golden = fixtures().join("golden").join(format!("{name}.json"));
    let actual = serde_json::to_string_pretty(report).unwrap() + "\n";

    if std::env::var_os("UPDATE_GOLDEN").is_some() {
        std::fs::write(&golden, actual).unwrap();
        return;
    }

    let expected = std::fs::read_to_string(&golden).unwrap();
    assert_eq!(actual, expected, "{name}: the report differs from {}", golden.display());
}
