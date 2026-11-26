use std::sync::Arc;
use std::sync::mpsc::Receiver;
use std::time::Duration;

use fresher_core::{Event, SettingUpdate, SettingValue};

use crate::Engine;
use crate::testing::{Fixture, HeldVolumes, ScriptedWalk, scan, wait_for_finish};

fn change_roots(engine: &Engine) {
    let roots = SettingValue::Folders(vec!["~/Projects".into()]);
    engine.update_setting(SettingUpdate { key: "core.scan.roots".into(), value: roots }).unwrap();
}

fn settings_changed(engine: &Engine) -> bool {
    engine.overview().unwrap().last_scan.unwrap().settings_changed
}

/// Waits until the scan has begun, so that a change made next is a change during it.
fn wait_for_start(events: &Receiver<Event>) {
    while !matches!(events.recv_timeout(Duration::from_secs(10)).unwrap(), Event::ScanStarted { .. }) {}
}

mod negative {
    use super::*;

    #[test]
    fn a_scan_with_no_change_ends_unmarked() {
        let (walk, gate) = ScriptedWalk::held();
        let engine = Fixture::new(walk).engine();
        let events = engine.subscribe();
        engine.start_scan(crate::ScanMode::Auto).unwrap();
        wait_for_start(&events);

        gate.release();
        wait_for_finish(&events);

        assert!(!settings_changed(&engine));
    }

    #[test]
    fn a_change_before_the_scan_does_not_mark_the_new_scan() {
        let engine = Fixture::new(ScriptedWalk::new()).engine();
        scan(&engine);
        change_roots(&engine);
        assert!(settings_changed(&engine));

        scan(&engine);

        assert!(!settings_changed(&engine));
    }

    #[test]
    fn a_change_of_a_setting_that_applies_at_once_does_not_mark_the_scan() {
        let (walk, gate) = ScriptedWalk::held();
        let engine = Fixture::new(walk).engine();
        let events = engine.subscribe();
        engine.start_scan(crate::ScanMode::Auto).unwrap();
        wait_for_start(&events);

        engine
            .update_setting(SettingUpdate { key: "core.motion".into(), value: SettingValue::Choice("reduced".into()) })
            .unwrap();
        gate.release();
        wait_for_finish(&events);

        assert!(!settings_changed(&engine));
    }
}

mod positive {
    use super::*;

    #[test]
    fn a_change_during_the_scan_marks_the_scan_that_ends() {
        let (walk, gate) = ScriptedWalk::held();
        let engine = Fixture::new(walk).engine();
        let events = engine.subscribe();
        engine.start_scan(crate::ScanMode::Auto).unwrap();
        wait_for_start(&events);

        change_roots(&engine);
        gate.release();
        wait_for_finish(&events);

        assert!(settings_changed(&engine));
    }

    #[test]
    fn a_change_after_the_start_before_the_scan_begins_marks_the_scan_that_ends() {
        let (volumes, gate) = HeldVolumes::new(fresher_testing::FakeVolumes::default());
        let volumes = Arc::new(volumes);
        let engine = Fixture::new(ScriptedWalk::new()).volumes_port(Arc::clone(&volumes)).engine();
        let events = engine.subscribe();
        engine.start_scan(crate::ScanMode::Auto).unwrap();
        volumes.wait_until_holding();

        change_roots(&engine);
        gate.release();
        wait_for_finish(&events);

        assert!(settings_changed(&engine));
    }

    #[test]
    fn a_cancelled_scan_is_marked_like_any_other() {
        let (walk, _gate) = ScriptedWalk::held();
        let engine = Fixture::new(walk).engine();
        let events = engine.subscribe();
        engine.start_scan(crate::ScanMode::Auto).unwrap();
        wait_for_start(&events);

        change_roots(&engine);
        engine.cancel_scan().unwrap();
        wait_for_finish(&events);

        assert!(settings_changed(&engine));
    }

    #[test]
    fn the_next_scan_starts_unmarked_again() {
        let (walk, gate) = ScriptedWalk::held();
        let engine = Fixture::new(walk).engine();
        let events = engine.subscribe();
        engine.start_scan(crate::ScanMode::Auto).unwrap();
        wait_for_start(&events);
        change_roots(&engine);
        gate.release();
        wait_for_finish(&events);

        engine.start_scan(crate::ScanMode::Auto).unwrap();
        wait_for_finish(&events);

        assert!(!settings_changed(&engine));
    }
}
