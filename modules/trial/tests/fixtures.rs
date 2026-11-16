//! The trial module on its fixture homes: what it must not claim first, then what it must, and its golden report.
//!
//! `UPDATE_GOLDEN=1 cargo nextest run -p fresher-module-trial` rewrites `fixtures/golden/trial.json`; the diff is
//! reviewed.

#[cfg(test)]
mod support;

use std::path::Path;

use fresher_core::NewFinding;
use fresher_testing::{FixtureTree, ModuleRun};

/// The finding whose identity is `path` under the tree's root, if any.
fn finding_at<'a>(run: &'a ModuleRun, tree: &FixtureTree, path: &str) -> Option<&'a NewFinding> {
    let path = tree.root().join(path);
    run.findings.iter().find(|finding| Path::new(finding.key().identity()) == path)
}

mod negative {
    use fresher_core::{ModuleOutcome, NewFinding};

    use crate::finding_at;
    use crate::support::{failed_listing, unclaimed};

    #[test]
    fn a_look_alike_name_is_not_claimed() {
        let (tree, run) = unclaimed();

        for path in
            ["look-alike/fresher-trial-cache2", "look-alike/fresher-trial-cache.txt", "look-alike/fresher-trial-kept2"]
        {
            assert_eq!(finding_at(&run, &tree, path), None, "{path}");
        }
    }

    #[test]
    fn a_file_with_the_name_is_not_claimed() {
        let (tree, run) = unclaimed();

        for path in ["file/fresher-trial-cache", "file/fresher-trial-kept"] {
            assert_eq!(finding_at(&run, &tree, path), None, "{path}");
        }
    }

    #[test]
    fn a_link_named_like_the_folder_is_not_claimed() {
        let (tree, run) = unclaimed();

        for path in ["link/fresher-trial-cache", "link/fresher-trial-kept", "outside/data"] {
            assert_eq!(finding_at(&run, &tree, path), None, "{path}");
        }
    }

    #[test]
    fn a_folder_without_the_sandbox_marker_next_to_it_is_not_claimed() {
        let (tree, run) = unclaimed();

        for path in ["unmarked/fresher-trial-cache", "marked-above/deeper/fresher-trial-kept"] {
            assert_eq!(finding_at(&run, &tree, path), None, "{path}");
        }
        assert_eq!(run.findings, Vec::<NewFinding>::new());
        assert_eq!(run.outcome, ModuleOutcome::Completed);
    }

    #[test]
    fn a_tool_that_cannot_list_its_objects_fails_the_module_alone() {
        let run = failed_listing();

        assert!(
            matches!(&run.outcome, ModuleOutcome::Failed { reason } if reason.key().as_str() == "trial.error.failed")
        );
    }
}

mod positive {
    use fresher_core::{DangerLevel, Removal, Subject};

    use crate::finding_at;
    use crate::support::{fixtures, sandbox};

    #[test]
    fn a_trial_cache_folder_is_claimed_whole() {
        let (tree, run) = sandbox();
        let path = tree.root().join("sandbox/fresher-trial-cache");

        let finding = finding_at(&run, &tree, "sandbox/fresher-trial-cache").unwrap();

        assert_eq!(finding.category().as_str(), "trial.caches");
        assert_eq!(finding.danger(), DangerLevel::Safe);
        assert_eq!(finding.subject(), &Subject::Path(path.clone()));
        assert_eq!(finding.removal(), &Removal::MoveToTrash(path));
        assert_eq!(finding.sizes().logical.bytes(), 1_048_576 + 2048);
        assert_eq!(finding_at(&run, &tree, "sandbox/fresher-trial-cache/nested"), None);
        assert_eq!(finding_at(&run, &tree, "sandbox/keep-me.txt"), None);
    }

    #[test]
    fn a_kept_folder_is_emptied_not_removed() {
        let (tree, run) = sandbox();
        let path = tree.root().join("sandbox/fresher-trial-kept");

        let finding = finding_at(&run, &tree, "sandbox/fresher-trial-kept").unwrap();

        assert_eq!(finding.category().as_str(), "trial.kept");
        assert_eq!(finding.danger(), DangerLevel::Low);
        assert_eq!(finding.removal(), &Removal::EmptyFolder(path));
    }

    #[test]
    fn the_tool_object_is_removed_with_a_fixed_argument_list() {
        let (_tree, run) = sandbox();

        let finding = run.findings.iter().find(|finding| finding.category().as_str() == "trial.tool").unwrap();

        assert_eq!(finding.danger(), DangerLevel::Medium);
        assert!(matches!(finding.subject(), Subject::External { tool, id }
            if tool.as_str() == "fresher-trial-tool" && id == "trial-object"));
        assert!(matches!(finding.removal(), Removal::ToolCommand(command)
            if command.tool().as_str() == "fresher-trial-tool" && command.args() == ["remove", "trial-object"]));
        assert_eq!(run.findings.len(), 3);
    }

    #[test]
    fn the_report_matches_the_golden_file() {
        let (_tree, run) = sandbox();
        let golden = fixtures().join("golden/trial.json");

        let report = run.golden().unwrap();

        if std::env::var_os("UPDATE_GOLDEN").is_some() {
            std::fs::write(&golden, &report).unwrap();
        }
        assert_eq!(report, std::fs::read_to_string(&golden).unwrap());
    }
}
