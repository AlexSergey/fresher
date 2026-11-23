//! The agreement before the first cleanup, end to end on files: the cleanup home `fixtures/homes/cleanup.toml` and the
//! uninstall sandbox, each with the real settings file in its data folder. Before the acceptance every prepare is
//! refused and nothing moves; an acceptance of another version is refused and not saved; the acceptance survives a
//! restart and a reset of the settings; put back never asks for it.

#[cfg(test)]
#[expect(dead_code, reason = "the agreement flows compare no golden file")]
mod support;

#[cfg(test)]
#[path = "cleaning/mod.rs"]
#[expect(dead_code, reason = "the cleanup home is shared; the agreement flows use its scan, plan and cleanup only")]
mod cleaning;

#[cfg(test)]
#[path = "uninstall_home/mod.rs"]
#[expect(dead_code, reason = "the uninstall home is shared; the agreement flows prepare one uninstall only")]
#[expect(clippy::duplicate_mod, reason = "both shared homes load the support's events and report helpers")]
mod flow;

#[cfg(test)]
mod checks {
    use std::fs;

    use fresher_core::{SettingsStore as _, StoredSettings};
    use fresher_store::{DataFolder, FileSettingsStore};
    use fresher_testing::TreeReport;

    use crate::cleaning::Home;

    /// Every entry of the uninstall sandbox: its apps folder, its home and its Trash.
    pub(crate) fn sandbox_tree(home: &crate::flow::Home) -> TreeReport {
        TreeReport::of(&home.fixture).unwrap()
    }

    /// The settings file of `home`'s data folder as written on disk.
    pub(crate) fn settings_file(home: &Home) -> String {
        fs::read_to_string(home.data.path().join("settings.json")).unwrap()
    }

    /// Saves `home`'s settings again without the acceptance of the agreement, as a file that never held it.
    pub(crate) fn forget_the_acceptance(home: &Home) {
        let store = FileSettingsStore::new(DataFolder::open(home.data.path()).unwrap());
        let StoredSettings { mut values } = store.load().unwrap();
        values.retain(|key, _| !key.starts_with("core.agreement."));
        store.save(&StoredSettings { values }).unwrap();
    }
}

mod negative {
    use std::fs;

    use fresher_core::{AGREEMENT_VERSION, BlockedReason, CommandError, UninstallChoice};

    use crate::checks::{sandbox_tree, settings_file};
    use crate::cleaning::{Setup, home};
    use crate::flow::{self, Home as UninstallHome};

    const REFUSED: CommandError = CommandError::CleaningBlocked { reason: BlockedReason::Agreement };

    #[test]
    fn a_cleanup_before_the_agreement_is_refused_and_nothing_moves() {
        let home = home(Setup { accept: false, ..Setup::DEFAULT });
        home.scan();
        let before = home.tree();

        let error = home.engine.prepare_cleanup(&home.selection());

        assert_eq!(error, Err(REFUSED));
        assert_eq!(home.tree(), before);
        assert!(!home.path("trash").exists());
        assert_eq!(home.engine.cleanups(0, 10).total, 0);
    }

    #[test]
    fn an_acceptance_of_another_version_is_refused_and_not_saved() {
        let home = home(Setup { accept: false, ..Setup::DEFAULT });

        let error = home.engine.accept_agreement(AGREEMENT_VERSION + 1);

        assert_eq!(error, Err(CommandError::AgreementChanged));
        assert!(!settings_file(&home).contains("core.agreement"), "{}", settings_file(&home));
        assert_eq!(home.engine.agreement().map(|view| view.accepted), Ok(None));
    }

    #[test]
    fn an_acceptance_over_a_broken_settings_file_is_refused_and_the_file_is_kept() {
        let setup = flow::Setup { accept: false, settings_file: Some("{ not json"), ..flow::Setup::default() };
        let home = UninstallHome::with(setup);
        let file = home.sandbox.data_folder().join("settings.json");

        let error = home.engine.accept_agreement(AGREEMENT_VERSION);

        assert_eq!(error, Err(CommandError::CleaningBlocked { reason: BlockedReason::Settings }));
        assert_eq!(fs::read_to_string(file).unwrap(), "{ not json");
        assert_eq!(home.engine.agreement().map(|view| view.accepted), Ok(None));
    }

    #[test]
    fn an_uninstall_before_the_agreement_is_refused_and_nothing_moves() {
        let home = UninstallHome::with(flow::Setup { accept: false, ..flow::Setup::default() });
        home.scan();
        let before = sandbox_tree(&home);

        let error = home.uninstall("Applications/Editor.app", UninstallChoice::Everything);

        assert_eq!(error, Err(REFUSED));
        assert_eq!(sandbox_tree(&home), before);
        assert!(home.path("Applications/Editor.app/Contents/Info.plist").exists());
        assert_eq!(home.engine.cleanups(0, 10).total, 0);
    }
}

mod positive {
    use fresher_core::{AGREEMENT_VERSION, BlockedReason, CommandError, Event, PutBackOutcome, TrashState};

    use serde_json::json;

    use crate::checks::{forget_the_acceptance, settings_file};
    use crate::cleaning::{Setup, home};

    /// The engine starts again and reads what the data folder holds; it accepts nothing itself.
    const RESTART: Setup = Setup { accept: false, ..Setup::DEFAULT };

    #[test]
    fn the_acceptance_survives_a_restart() {
        let home = home(RESTART);
        let accepted = home.engine.accept_agreement(AGREEMENT_VERSION).unwrap();

        let home = home.restarted(RESTART);

        assert!(accepted.is_accepted());
        assert_eq!(home.engine.agreement(), Ok(accepted));
        home.scan();
        let view = home.prepare();
        let (_, events) = home.clean(&view, "trash");
        assert!(matches!(events.last(), Some(Event::CleanupFinished { .. })));
        assert!(!home.path("home/sandbox/fresher-trial-cache").exists());
        assert!(home.path("trash").read_dir().unwrap().next().is_some());
        assert_eq!(home.engine.cleanups(0, 10).total, 1);
    }

    #[test]
    fn a_reset_of_the_settings_keeps_the_acceptance() {
        let home = home(RESTART);
        let accepted = home.engine.accept_agreement(AGREEMENT_VERSION).unwrap();

        home.engine.reset_settings().unwrap();

        assert_eq!(home.engine.agreement(), Ok(accepted.clone()));
        assert!(settings_file(&home).contains("core.agreement.version"), "{}", settings_file(&home));
        let home = home.restarted(RESTART);
        assert_eq!(home.engine.agreement(), Ok(accepted));
    }

    #[test]
    fn put_back_works_without_the_acceptance() {
        let home = home(Setup::DEFAULT);
        home.scan();
        let view = home.prepare();
        let (cleanup, _) = home.clean(&view, "trash");
        let cache = home.path("home/sandbox/fresher-trial-cache");
        let detail = home.engine.cleanup(cleanup).unwrap();
        let item = detail.items.iter().find(|item| item.path.as_deref() == Some(cache.as_path())).unwrap();
        assert_eq!(item.trash, Some(TrashState::InTrash));
        let index = item.index;
        forget_the_acceptance(&home);

        let home = home.restarted(RESTART);
        let back = home.engine.put_back(cleanup, Some(&[index])).unwrap();

        assert!(!home.engine.agreement().unwrap().is_accepted());
        assert_eq!(back.items[0].outcome, PutBackOutcome::PutBack);
        assert!(home.path("home/sandbox/fresher-trial-cache/nested/blob.bin").exists());
        home.scan();
        let caches = json!({ "findings": [], "categories": [{ "category": "trial.caches", "except": [] }] });
        let refused = CommandError::CleaningBlocked { reason: BlockedReason::Agreement };
        assert_eq!(home.engine.prepare_cleanup(&serde_json::from_value(caches).unwrap()), Err(refused));
    }
}
