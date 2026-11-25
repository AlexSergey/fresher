//! The High switch end to end on files: the app-leftovers home `fixtures/homes/app-leftovers-flow.toml` (its removed
//! app's data is High, its caches Safe), with the real scanner, settings file, journal and `DiskCleaner`, with a fake
//! Trash. Withheld, no High finding is answered, counted or explained by the storage map, and every way to remove one
//! is refused, acting on nothing; offered, the same home lists it as before.

#[cfg(test)]
#[expect(dead_code, reason = "the High switch flows compare no golden file")]
mod support;

#[cfg(test)]
#[path = "support/tree.rs"]
mod tree;

#[cfg(test)]
#[path = "app_leftovers_home/mod.rs"]
#[expect(dead_code, reason = "the app-leftovers home is shared; these flows scan it and prepare from it only")]
mod flow;

#[cfg(test)]
mod checks {
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    use fresher_core::{CleanupSelection, HighLevel, PartKind, StorageMap};
    use fresher_sandbox::Sandbox;
    use serde_json::json;

    use crate::flow::home_engine::{Extras, engine_with};
    use crate::flow::{Flow, Setup, parts};
    use crate::tree::contents;

    /// The removed app's data, High.
    pub(crate) const DATA: &str = "app-leftovers.data";
    /// The removed app's caches, Safe.
    pub(crate) const CACHES: &str = "app-leftovers.caches";

    /// The home, its engine built with `high`, scanned once.
    pub(crate) fn scanned(high: HighLevel) -> Flow {
        let mut flow = Flow::new();
        let extras = Extras { high, ..Extras::default() };
        flow.engine =
            engine_with(&flow.sandbox, &flow.stated, &flow.open_files, parts(&flow.sandbox, Setup::default()), extras);
        flow.scan();
        flow
    }

    /// `category` picked whole.
    pub(crate) fn whole(category: &str) -> CleanupSelection {
        serde_json::from_value(json!({ "findings": [], "categories": [{ "category": category, "except": [] }] }))
            .unwrap()
    }

    /// The allocated bytes the storage map explains ("Can be freed") on its only volume.
    pub(crate) fn explained(map: &StorageMap) -> u64 {
        let parts = map.volumes[0].parts().iter();
        parts.filter(|part| part.kind == PartKind::Explained).map(|part| part.sizes.allocated.bytes()).sum()
    }

    /// The allocated bytes of the overview's total of `category`; nothing when it has none.
    pub(crate) fn total(flow: &Flow, category: &str) -> u64 {
        let overview = flow.engine.overview().unwrap();
        let total = overview.totals.iter().find(|total| total.category.as_str() == category);
        total.map_or(0, |total| total.sizes.allocated.bytes())
    }

    /// Every file of the sandbox's home and its Trash, with its bytes.
    pub(crate) fn every_file(sandbox: &Sandbox) -> BTreeMap<PathBuf, Vec<u8>> {
        [sandbox.home(), sandbox.trash()]
            .iter()
            .filter(|folder| folder.exists())
            .flat_map(|folder| contents(folder))
            .collect()
    }
}

mod negative {
    use fresher_core::{CommandError, DangerLevel, HighLevel};

    use crate::checks::{CACHES, DATA, every_file, explained, scanned, total, whole};

    #[test]
    fn a_withheld_home_offers_no_high_finding_and_refuses_its_plan() {
        let offered = scanned(HighLevel::Offered);
        let withheld = scanned(HighLevel::Withheld);
        let before = every_file(&withheld.sandbox);

        let found = withheld.findings();
        assert!(found.iter().any(|(category, _)| category == CACHES), "{found:?}");
        assert!(found.iter().all(|(category, _)| category != DATA), "{found:?}");
        let overview = withheld.engine.overview().unwrap();
        assert!(overview.totals.iter().all(|total| total.highest_danger < DangerLevel::High), "{overview:?}");
        assert_eq!(total(&withheld, DATA), 0);
        assert_eq!(total(&withheld, CACHES), total(&offered, CACHES));
        let data = total(&offered, DATA);
        assert!(data > 0);
        assert_eq!(overview.found.allocated.bytes(), offered.engine.overview().unwrap().found.allocated.bytes() - data);
        let (offered_map, withheld_map) =
            (offered.engine.storage_map().unwrap(), withheld.engine.storage_map().unwrap());
        assert_eq!(explained(&withheld_map), explained(&offered_map) - data);
        let module =
            withheld.engine.modules().into_iter().find(|module| module.id.as_str() == "app-leftovers").unwrap();
        assert!(module.categories.iter().all(|category| category.descriptor.id().as_str() != DATA));
        assert!(module.choices.iter().all(|choice| choice.categories.iter().all(|id| id.as_str() != DATA)));

        assert_eq!(withheld.engine.prepare_cleanup(&whole(DATA)), Err(CommandError::NothingToClean));
        let state = withheld.engine.disk_state().unwrap().unwrap();
        let root = state.roots()[0];
        assert_eq!(withheld.engine.pick_own_item(root), Err(CommandError::HighWithheld));

        assert_eq!(every_file(&withheld.sandbox), before);
        assert_eq!(withheld.engine.cleanups(0, 10).total, 0);
    }
}

mod positive {
    use fresher_core::{DangerLevel, HighLevel};

    use crate::checks::{DATA, scanned, whole};

    #[test]
    fn the_same_home_offered_lists_the_high_finding() {
        let offered = scanned(HighLevel::Offered);

        assert!(offered.findings().iter().any(|(category, _)| category == DATA));
        let view = offered.engine.prepare_cleanup(&whole(DATA)).unwrap();
        assert_eq!(view.summary.highest, DangerLevel::High);
        assert_eq!(offered.engine.high_level(), HighLevel::Offered);
    }
}
