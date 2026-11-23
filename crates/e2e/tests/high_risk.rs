//! The tick a High plan needs, end to end on files: the downloads home `fixtures/homes/downloads-flow.toml` (its
//! large untouched file is High) and the uninstall sandbox (an uninstall of everything is High), each with the real
//! scanner, settings file, journal and `DiskCleaner`, with a fake Trash. Confirmed without the user's tick that the
//! items may be the only copy, the plan is refused and nothing moves; with it, the cleanup goes to the Trash and comes
//! back by put back. A plan below High needs no tick.

#[cfg(test)]
#[expect(dead_code, reason = "the high-risk flows compare no golden file")]
mod support;

#[cfg(test)]
#[path = "support/tree.rs"]
mod tree;

#[cfg(test)]
#[path = "downloads_home/mod.rs"]
#[expect(dead_code, reason = "the downloads home is shared; the high-risk flows clean its large file only")]
mod downloads;

#[cfg(test)]
#[path = "uninstall_home/mod.rs"]
#[expect(dead_code, reason = "the uninstall home is shared; the high-risk flows prepare one uninstall each")]
#[expect(clippy::duplicate_mod, reason = "both shared homes load the support's events and report helpers")]
mod uninstall;

#[cfg(test)]
mod checks {
    use std::collections::BTreeMap;
    use std::path::PathBuf;
    use std::sync::mpsc::Receiver;
    use std::time::Duration;

    use fresher_core::{CleanupPlanView, CleanupSelection, Confirmation, Event};
    use fresher_sandbox::Sandbox;
    use serde_json::json;

    use crate::tree::contents;

    /// The category of the large untouched files, High.
    pub(crate) const LARGE: &str = "downloads.large-untouched";
    /// The large untouched file of the downloads home.
    pub(crate) const VIDEO: &str = "home/Downloads/video.mov";
    pub(crate) const BUNDLE: &str = "Applications/Editor.app";

    /// The large untouched files picked whole.
    pub(crate) fn large() -> CleanupSelection {
        serde_json::from_value(json!({ "findings": [], "categories": [{ "category": LARGE, "except": [] }] })).unwrap()
    }

    /// The confirmation of `view` to the Trash as the dialog shows it, the High tick as given.
    pub(crate) fn echo(view: &CleanupPlanView, high_risk_understood: bool) -> Confirmation {
        let summary = view.summary;
        let echo = json!({ "plan": view.plan, "count": summary.count, "allocated": summary.sizes.allocated,
                           "highest": summary.highest, "method": "trash", "highRiskUnderstood": high_risk_understood });
        serde_json::from_value(echo).unwrap()
    }

    /// Every file of the home, its Trash and the apps folder, with its bytes.
    pub(crate) fn every_file(sandbox: &Sandbox) -> BTreeMap<PathBuf, Vec<u8>> {
        let folders = [sandbox.home(), sandbox.trash(), sandbox.path().join("Applications")];
        folders.iter().filter(|folder| folder.exists()).flat_map(|folder| contents(folder)).collect()
    }

    /// Waits for the end of the cleanup; fails after 30 s instead of hanging.
    pub(crate) fn wait_for_the_end(events: &Receiver<Event>) {
        while !matches!(events.recv_timeout(Duration::from_secs(30)).unwrap(), Event::CleanupFinished { .. }) {}
    }
}

mod negative {
    use fresher_core::{CommandError, DangerLevel, UninstallChoice};

    use crate::checks::{BUNDLE, VIDEO, echo, every_file, large};
    use crate::{downloads, uninstall};

    #[test]
    fn a_high_cleanup_without_the_tick_moves_nothing() {
        let home = downloads::Home::new();
        home.opt_in();
        home.flow.scan();
        let view = home.flow.engine.prepare_cleanup(&large()).unwrap();
        let before = every_file(&home.flow.sandbox);

        let error = home.flow.engine.confirm_cleanup(&echo(&view, false));

        assert_eq!(view.summary.highest, DangerLevel::High);
        assert_eq!(error, Err(CommandError::HighRiskNotUnderstood));
        assert_eq!(every_file(&home.flow.sandbox), before);
        assert!(home.flow.path(VIDEO).is_file());
        assert_eq!(home.flow.engine.cleanups(0, 10).total, 0);
        assert_eq!(home.flow.engine.confirm_cleanup(&echo(&view, true)), Err(CommandError::PlanGone));
    }

    #[test]
    fn an_uninstall_of_everything_without_the_tick_moves_nothing() {
        let home = uninstall::Home::new();
        home.scan();
        let view = home.uninstall(BUNDLE, UninstallChoice::Everything).unwrap();
        let before = every_file(&home.flow.sandbox);

        let error = home.engine.confirm_cleanup(&echo(&view, false));

        assert_eq!(view.summary.highest, DangerLevel::High);
        assert_eq!(error, Err(CommandError::HighRiskNotUnderstood));
        assert_eq!(every_file(&home.flow.sandbox), before);
        assert!(home.app_id(BUNDLE).is_some(), "the app is still listed");
        assert_eq!(home.engine.cleanups(0, 10).total, 0);
    }
}

mod positive {
    use std::fs;

    use fresher_core::{DangerLevel, PutBackOutcome, UninstallChoice};

    use crate::checks::{BUNDLE, VIDEO, echo, large, wait_for_the_end};
    use crate::{downloads, uninstall};

    #[test]
    fn a_high_cleanup_with_the_tick_goes_to_the_trash_and_comes_back_by_put_back() {
        let home = downloads::Home::new();
        home.opt_in();
        home.flow.scan();
        let view = home.flow.engine.prepare_cleanup(&large()).unwrap();
        let bytes = fs::read(home.flow.path(VIDEO)).unwrap();
        let events = home.flow.engine.subscribe();

        let cleanup = home.flow.engine.confirm_cleanup(&echo(&view, true)).unwrap();
        wait_for_the_end(&events);

        assert_eq!(view.summary.highest, DangerLevel::High);
        assert!(!home.flow.path(VIDEO).exists());
        assert_eq!(fs::read_dir(home.flow.sandbox.trash()).unwrap().count(), 1);
        let back = home.flow.engine.put_back(cleanup, None).unwrap();
        assert!(back.items.iter().all(|item| item.outcome == PutBackOutcome::PutBack), "{back:?}");
        assert_eq!(fs::read(home.flow.path(VIDEO)).unwrap(), bytes, "put back byte for byte");
    }

    #[test]
    fn an_uninstall_that_keeps_the_data_needs_no_tick() {
        let home = uninstall::Home::new();
        home.scan();
        let view = home.uninstall(BUNDLE, UninstallChoice::KeepData).unwrap();
        let events = home.engine.subscribe();

        home.engine.confirm_cleanup(&echo(&view, false)).unwrap();
        wait_for_the_end(&events);

        assert_eq!(view.summary.highest, DangerLevel::Medium);
        assert!(!home.path(BUNDLE).exists());
        assert_eq!(home.app_id(BUNDLE), None, "the app leaves the list");
    }
}
