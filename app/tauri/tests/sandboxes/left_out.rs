//! The left-out check of a case file: what the scan's modules left out, counted per category and reason, equals what
//! the cases' `[[case.left-out]]` expect, exactly, in both directions.

use std::collections::BTreeMap;
use std::path::Path;

use fresher_core::LeftOutCount;
use fresher_sandbox::CaseFile;
use serde_json::Value;

use crate::harness::{Failure, failure, relative};

/// Items per (category id, reason key).
type Counts = BTreeMap<(String, String), u32>;

/// The scan's left-outs sorted by category, then reason; one module's category is in one module's report only, so a
/// pair is never counted twice.
pub(crate) fn sorted(reported: &[LeftOutCount]) -> Vec<LeftOutCount> {
    let mut counts = reported.to_vec();
    counts.sort_by(|a, b| (a.category.as_str(), a.reason.key()).cmp(&(b.category.as_str(), b.reason.key())));
    counts
}

/// Fails when the file's expectations and the scan's left-outs differ, naming every pair that does.
pub(crate) fn check_left_outs(name: &str, file: &CaseFile, reported: &[LeftOutCount]) -> Result<(), Failure> {
    let expected: Counts = file
        .left_outs()
        .iter()
        .map(|sum| ((sum.category().as_str().to_owned(), sum.reason().as_str().to_owned()), sum.items()))
        .collect();
    let reported: Counts = reported
        .iter()
        .map(|count| ((count.category.as_str().to_owned(), count.reason.key().as_str().to_owned()), count.items))
        .collect();
    compare(&expected, &reported).map_err(|problem| failure(name, problem))
}

/// The pairs whose counts differ, each on its own, sorted by category then reason.
fn compare(expected: &Counts, reported: &Counts) -> Result<(), String> {
    let mut pairs: Vec<&(String, String)> = expected.keys().chain(reported.keys()).collect();
    pairs.sort();
    pairs.dedup();
    let problems: Vec<String> = pairs
        .into_iter()
        .filter_map(|pair| {
            let (category, reason) = pair;
            match (expected.get(pair), reported.get(pair)) {
                (Some(want), Some(got)) if want == got => None,
                (Some(want), Some(got)) => {
                    Some(format!("`{category}` left out for `{reason}`: expected {want} items, got {got}"))
                }
                (Some(want), None) => {
                    Some(format!("`{category}` left out for `{reason}`: expected {want} items, none"))
                }
                (None, Some(got)) => {
                    Some(format!("`{category}` left out for `{reason}`: {got} items, no case expects it"))
                }
                (None, None) => None,
            }
        })
        .collect();
    if problems.is_empty() { Ok(()) } else { Err(format!("left-outs differ: {}", problems.join("; "))) }
}

/// The report's `leftOut`: the counts as the engine made them, each listed item's path relative to the sandbox, `None`
/// when there are none, so a case file that leaves nothing out keeps the report it always had.
pub(crate) fn report(reported: &[LeftOutCount], sandbox: &Path) -> Option<Value> {
    (!reported.is_empty()).then(|| {
        let mut counts = serde_json::to_value(reported).unwrap();
        let shown = counts.as_array_mut().into_iter().flatten().filter_map(|count| count["shown"].as_array_mut());
        for (entry, count) in shown.flatten().zip(reported.iter().flat_map(|count| &count.shown)) {
            entry["path"] = Value::String(relative(&count.path, sandbox));
        }
        counts
    })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{Counts, compare};

    fn counts(entries: &[(&str, &str, u32)]) -> Counts {
        entries
            .iter()
            .map(|(category, reason, items)| ((category.to_string(), reason.to_string()), *items))
            .collect::<BTreeMap<_, _>>()
    }

    mod negative {
        use super::{compare, counts};

        #[test]
        fn a_left_out_no_case_expects_fails_naming_it() {
            let problem = compare(&counts(&[]), &counts(&[("demo.caches", "demo.left-out.open", 1)])).unwrap_err();

            assert!(problem.contains("`demo.caches` left out for `demo.left-out.open`: 1 items, no case expects it"));
        }

        #[test]
        fn an_expected_left_out_that_is_missing_fails_naming_it() {
            let problem = compare(&counts(&[("demo.caches", "demo.left-out.open", 2)]), &counts(&[])).unwrap_err();

            assert!(problem.contains("expected 2 items, none"), "{problem}");
        }

        #[test]
        fn a_count_that_differs_fails_with_both_numbers() {
            let expected = counts(&[("demo.caches", "demo.left-out.open", 2)]);

            let problem = compare(&expected, &counts(&[("demo.caches", "demo.left-out.open", 3)])).unwrap_err();

            assert!(problem.contains("expected 2 items, got 3"), "{problem}");
        }

        #[test]
        fn every_differing_pair_is_named_in_order() {
            let expected = counts(&[("b.x", "b.r", 1)]);
            let reported = counts(&[("a.x", "a.r", 1), ("b.x", "b.s", 1)]);

            let problem = compare(&expected, &reported).unwrap_err();

            let (a, b_r, b_s) =
                (problem.find("`a.x`").unwrap(), problem.find("`b.r`").unwrap(), problem.find("`b.s`").unwrap());
            assert!(a < b_r && b_r < b_s, "{problem}");
        }
    }

    mod positive {
        use super::{compare, counts};

        #[test]
        fn no_expectation_and_no_left_out_passes() {
            compare(&counts(&[]), &counts(&[])).unwrap();
        }

        #[test]
        fn equal_counts_pass() {
            let same = counts(&[("demo.caches", "demo.left-out.open", 1), ("trial.kept", "trial.error.failed", 4)]);

            compare(&same, &same.clone()).unwrap();
        }
    }
}
