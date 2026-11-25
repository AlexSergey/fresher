//! What a case file that says `high = "withheld"` proves whatever its cases say: the app, which withholds High, offers
//! nothing High. No finding is at High, no category total holds a High item, and the storage map's "Explained" holds
//! exactly what was found, so a High item is in none of them.

use fresher_core::{DangerLevel, Finding, HighLevel, PartKind, VolumeMap};
use fresher_engine::Engine;
use fresher_sandbox::CaseFile;

use crate::harness::{Failure, failure};

/// For a file run with High withheld: nothing of `findings`, the overview or the map holds a High item.
pub(crate) fn check_withheld(
    name: &str,
    file: &CaseFile,
    engine: &Engine,
    findings: &[Finding],
) -> Result<(), Failure> {
    if file.high() == HighLevel::Offered {
        return Ok(());
    }
    let overview = engine.overview().map_err(|error| failure(name, format!("the overview: {error:?}")))?;
    let map = engine.storage_map().map_err(|error| failure(name, format!("the map: {error:?}")))?;
    let explained = map
        .volumes
        .iter()
        .flat_map(VolumeMap::parts)
        .filter(|part| matches!(part.kind, PartKind::Explained))
        .map(|part| part.sizes.allocated.bytes())
        .sum();
    let found: Vec<_> =
        findings.iter().map(|finding| (finding.finding().category().as_str(), finding.finding().danger())).collect();
    let totals: Vec<_> = overview.totals.iter().map(|total| (total.category.as_str(), total.highest_danger)).collect();
    match high_item(&found, &totals, explained, overview.found.allocated.bytes()) {
        None => Ok(()),
        Some(problem) => Err(failure(name, format!("High is withheld, yet {problem}"))),
    }
}

/// What holds a High item: a finding at High, a category total whose highest level is High (each by its category),
/// or an "Explained" part of other bytes than were found.
fn high_item(
    findings: &[(&str, DangerLevel)],
    totals: &[(&str, DangerLevel)],
    explained: u64,
    found: u64,
) -> Option<String> {
    if let Some((category, _)) = findings.iter().find(|(_, danger)| *danger == DangerLevel::High) {
        return Some(format!("a finding of {category} is at High"));
    }
    if let Some((category, _)) = totals.iter().find(|(_, danger)| *danger == DangerLevel::High) {
        return Some(format!("the total of {category} holds a High item"));
    }
    (explained != found).then(|| format!("the map explains {explained} bytes, {found} were found"))
}

#[cfg(test)]
mod tests {
    mod negative {
        use fresher_core::DangerLevel;

        use crate::withheld::high_item;

        #[test]
        fn a_finding_at_high_is_named() {
            let findings = [("app-leftovers.caches", DangerLevel::Safe), ("app-leftovers.data", DangerLevel::High)];

            let problem = high_item(&findings, &[], 0, 0);

            assert_eq!(problem.as_deref(), Some("a finding of app-leftovers.data is at High"));
        }

        #[test]
        fn a_total_holding_a_high_item_is_named() {
            let problem = high_item(&[], &[("app-leftovers.settings", DangerLevel::High)], 0, 0);

            assert_eq!(problem.as_deref(), Some("the total of app-leftovers.settings holds a High item"));
        }

        #[test]
        fn an_explained_part_of_other_bytes_than_were_found_is_named() {
            let problem = high_item(&[], &[], 8192, 4096);

            assert_eq!(problem.as_deref(), Some("the map explains 8192 bytes, 4096 were found"));
        }
    }

    mod positive {
        use fresher_core::DangerLevel;

        use crate::withheld::high_item;

        #[test]
        fn nothing_at_high_holds() {
            let findings =
                [("app-leftovers.caches", DangerLevel::Safe), ("app-leftovers.settings", DangerLevel::Medium)];
            let totals = [("app-leftovers.caches", DangerLevel::Safe), ("app-leftovers.settings", DangerLevel::Medium)];

            assert_eq!(high_item(&findings, &totals, 4096, 4096), None);
        }
    }
}
