//! The scenario's format, its refusals and the shape the UI receives.

use super::{Scenario, ScenarioError};

const WINDOW: &str = "world = \"developer-mac\"\nwindow = { width = 1100, height = 720 }\n";

/// A scenario of the window above and `steps`.
fn parse(steps: &str) -> Result<Scenario, ScenarioError> {
    Scenario::parse("overview", &format!("{WINDOW}{steps}"))
}

/// The error of a scenario of `steps`, with its message.
fn refused(steps: &str) -> (ScenarioError, String) {
    let error = parse(steps).unwrap_err();
    let message = error.to_string();
    (error, message)
}

mod negative {
    use super::{WINDOW, refused};
    use crate::{Scenario, ScenarioError};

    #[test]
    fn an_unknown_key_is_refused() {
        for text in [
            format!("{WINDOW}colour = 1\n[[step]]\npause = 1\n"),
            format!("{WINDOW}[[step]]\npause = 1\nspeed = 2\n"),
            "world = \"developer-mac\"\nwindow = { width = 1100, height = 720, depth = 1 }\n".into(),
        ] {
            let error = Scenario::parse("overview", &text).unwrap_err();
            assert!(matches!(error, ScenarioError::Format { .. }), "{text}: {error}");
        }
    }

    #[test]
    fn an_unknown_role_is_refused() {
        let (error, _) = refused("[[step]]\nclick = { role = \"menu\", name = \"Scan\" }\n");

        assert!(matches!(error, ScenarioError::Format { .. }), "{error}");
    }

    #[test]
    fn a_world_that_is_not_a_plain_name_is_refused() {
        for world in ["../x", "", "developer mac", "a/b", "x.toml"] {
            let text = WINDOW.replace("developer-mac", world) + "[[step]]\npause = 1\n";

            let error = Scenario::parse("overview", &text).unwrap_err();

            assert!(matches!(error, ScenarioError::BadWorld { .. }), "{world:?}: {error}");
        }
    }

    #[test]
    fn a_window_out_of_bounds_is_refused() {
        for (width, height) in [(800, 720), (1601, 720), (1100, 599), (1100, 1001)] {
            let text = format!(
                "world = \"developer-mac\"\nwindow = {{ width = {width}, height = {height} }}\n[[step]]\npause = 1\n"
            );

            let error = Scenario::parse("overview", &text).unwrap_err();

            assert!(matches!(error, ScenarioError::Window { .. }), "{width} × {height}: {error}");
            assert!(error.to_string().contains(&format!("{width} × {height}")), "{error}");
        }
    }

    #[test]
    fn a_scenario_without_steps_is_refused() {
        let (error, _) = refused("");

        assert!(matches!(error, ScenarioError::NoSteps), "{error}");
    }

    #[test]
    fn a_step_with_two_keys_or_none_is_refused_naming_it() {
        for steps in [
            "[[step]]\npause = 1\n[[step]]\nclick = { role = \"button\", name = \"Scan\" }\npause = 5\n",
            "[[step]]\npause = 1\n[[step]]\n",
        ] {
            let (error, message) = refused(steps);

            assert!(matches!(error, ScenarioError::StepKeys { step: 2 }), "{error}");
            assert!(message.contains("step 2"), "{message}");
        }
    }

    #[test]
    fn a_target_with_both_names_neither_or_an_empty_one_is_refused_naming_its_step() {
        for target in [
            "{ role = \"button\", name = \"Scan\", name-start = \"Sc\" }",
            "{ role = \"button\" }",
            "{ role = \"button\", name = \"\" }",
            "{ role = \"button\", name-start = \"  \" }",
        ] {
            for key in ["click", "hover", "wait"] {
                let (error, message) = refused(&format!("[[step]]\npause = 1\n[[step]]\n{key} = {target}\n"));

                assert!(matches!(error, ScenarioError::Name { step: 2 }), "{key} = {target}: {error}");
                assert!(message.contains("step 2"), "{message}");
            }
        }
        let (error, _) = refused("[[step]]\nscroll = { role = \"region\", by = 10 }\n");
        assert!(matches!(error, ScenarioError::Name { step: 1 }), "{error}");
    }

    #[test]
    fn a_scroll_by_zero_is_refused_naming_its_step() {
        let (error, message) = refused("[[step]]\nscroll = { role = \"region\", name = \"Results\", by = 0 }\n");

        assert!(matches!(error, ScenarioError::ScrollZero { step: 1 }), "{error}");
        assert!(message.contains("step 1"), "{message}");
    }

    #[test]
    fn a_pause_out_of_bounds_is_refused_naming_its_step() {
        for ms in [0, 10_001] {
            let (error, message) = refused(&format!("[[step]]\npause = 1\n[[step]]\npause = {ms}\n"));

            assert!(matches!(error, ScenarioError::Pause { step: 2, .. }), "{ms}: {error}");
            assert!(message.contains("step 2"), "{message}");
        }
    }

    #[test]
    fn pauses_adding_up_to_more_than_twenty_seconds_are_refused() {
        let (error, message) = refused("[[step]]\npause = 10000\n[[step]]\npause = 10000\n[[step]]\npause = 1\n");

        assert!(matches!(error, ScenarioError::TooLong { total_ms: 20_001 }), "{error}");
        assert!(message.contains("20001"), "{message}");
    }

    #[test]
    fn a_missing_file_is_refused_naming_its_path() {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("gone.toml");

        let error = Scenario::read(&path).unwrap_err();

        assert!(matches!(error, ScenarioError::Io { .. }), "{error}");
        assert!(error.to_string().contains(&path.display().to_string()), "{error}");
    }
}

mod positive {
    use std::fs;
    use std::path::Path;

    use serde_json::json;

    use super::parse;
    use crate::{AppState, DemoOutcome, DemoWindow, NameMatch, Role, Scenario, Step, Target, Wait};

    fn target(role: Role, name: NameMatch) -> Target {
        Target { role, name }
    }

    fn exact(text: &str) -> NameMatch {
        NameMatch::Exact { text: text.into() }
    }

    /// The scenario of the design (docs/plans/038-demo-recordings/00-design.md, section 2).
    #[test]
    fn the_designs_example_parses_into_its_typed_steps() {
        let text = r#"
world = "developer-mac"
window = { width = 1100, height = 720 }

[[step]]
click = { role = "button", name = "Scan" }
[[step]]
wait = "scan-finished"
[[step]]
pause = 1200
[[step]]
hover = { role = "button", name-start = "Developer caches" }
[[step]]
scroll = { role = "region", name = "Results", by = 480 }
[[step]]
wait = { role = "heading", name = "Cleanups" }
"#;

        let scenario = Scenario::parse("overview", text).unwrap();

        assert_eq!(
            scenario,
            Scenario {
                name: "overview".into(),
                world: "developer-mac".into(),
                window: DemoWindow { width: 1100, height: 720 },
                steps: vec![
                    Step::Click { target: target(Role::Button, exact("Scan")) },
                    Step::Wait { until: Wait::State { state: AppState::ScanFinished } },
                    Step::Pause { ms: 1200 },
                    Step::Hover { target: target(Role::Button, NameMatch::Start { text: "Developer caches".into() }) },
                    Step::Scroll { target: target(Role::Region, exact("Results")), by: 480 },
                    Step::Wait { until: Wait::Target { target: target(Role::Heading, exact("Cleanups")) } },
                ],
            }
        );
    }

    #[test]
    fn every_role_and_state_is_read() {
        for (role, expected) in [
            ("button", Role::Button),
            ("link", Role::Link),
            ("tab", Role::Tab),
            ("checkbox", Role::Checkbox),
            ("switch", Role::Switch),
            ("heading", Role::Heading),
            ("region", Role::Region),
            ("dialog", Role::Dialog),
            ("row", Role::Row),
        ] {
            let scenario = parse(&format!("[[step]]\nclick = {{ role = \"{role}\", name = \"x\" }}\n")).unwrap();
            assert_eq!(scenario.steps, [Step::Click { target: target(expected, exact("x")) }], "{role}");
        }
        let scenario = parse("[[step]]\nwait = \"cleanup-finished\"\n[[step]]\nscroll = { role = \"row\", name-start = \"N\", by = -5 }\n").unwrap();
        assert_eq!(
            scenario.steps,
            [
                Step::Wait { until: Wait::State { state: AppState::CleanupFinished } },
                Step::Scroll { target: target(Role::Row, NameMatch::Start { text: "N".into() }), by: -5 },
            ]
        );
    }

    #[test]
    fn pauses_of_twenty_seconds_exactly_are_allowed() {
        let scenario = parse("[[step]]\npause = 10000\n[[step]]\npause = 10000\n").unwrap();

        assert_eq!(scenario.steps, [Step::Pause { ms: 10_000 }, Step::Pause { ms: 10_000 }]);
    }

    #[test]
    fn a_step_serializes_in_the_shape_the_ui_gets() {
        let steps = [
            Step::Click { target: target(Role::Button, exact("Scan")) },
            Step::Wait { until: Wait::State { state: AppState::ScanFinished } },
            Step::Scroll { target: target(Role::Region, NameMatch::Start { text: "Res".into() }), by: 480 },
            Step::Pause { ms: 800 },
        ];

        let json = serde_json::to_value(steps).unwrap();

        assert_eq!(
            json,
            json!([
                { "kind": "click", "target": { "role": "button", "name": { "kind": "exact", "text": "Scan" } } },
                { "kind": "wait", "until": { "kind": "state", "state": "scan-finished" } },
                {
                    "kind": "scroll",
                    "target": { "role": "region", "name": { "kind": "start", "text": "Res" } },
                    "by": 480
                },
                { "kind": "pause", "ms": 800 },
            ])
        );
    }

    #[test]
    fn an_outcome_reads_and_writes_in_camel_case() {
        let finished = DemoOutcome::Finished { elapsed_ms: 15_300 };
        let failed = DemoOutcome::Failed { step: 3, reason: "no target".into() };

        let json = [serde_json::to_value(&finished).unwrap(), serde_json::to_value(&failed).unwrap()];

        assert_eq!(json[0], json!({ "kind": "finished", "elapsedMs": 15_300 }));
        assert_eq!(json[1], json!({ "kind": "failed", "step": 3, "reason": "no target" }));
        assert_eq!(serde_json::from_value::<DemoOutcome>(json[0].clone()).unwrap(), finished);
        assert_eq!(serde_json::from_value::<DemoOutcome>(json[1].clone()).unwrap(), failed);
    }

    /// Every scenario of the repository parses and plays a demo world of `fixtures/demos/`.
    #[test]
    fn every_scenario_of_the_repository_parses_and_names_a_demo_world() {
        let demos = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/demos");
        let mut scenarios = 0;
        for entry in fs::read_dir(demos.join("scenarios")).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().is_none_or(|extension| extension != "toml") {
                continue;
            }
            let scenario = Scenario::read(&path).map_err(|error| format!("{}: {error}", path.display())).unwrap();
            let world = demos.join(format!("{}.toml", scenario.world));
            let file = crate::CaseFile::parse(&fs::read_to_string(&world).unwrap()).unwrap();
            assert!(file.demo().is_some(), "{}: {} is no demo world", path.display(), world.display());
            scenarios += 1;
        }
        assert!(scenarios > 0, "no scenario in {}", demos.display());
    }
}
