//! Every target of a scenario is traced to the text key the interface shows it under, and no step may tick or press the
//! permanent delete: a demo moves to the sandbox's Trash only.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use super::{NameMatch, Scenario, Step, Target, Wait};

/// The texts that name a control: the interface's, the core's and every module's.
type Dictionary = BTreeMap<String, String>;

/// Whether a key is one a demo never presses, as the UI's player refuses them at run time: every `ui.` key with
/// "permanent" in it, the Delete button and the tool removal (which cannot be put back).
fn refused_key(key: &str) -> bool {
    key.starts_with("ui.") && (key.contains("permanent") || key == "ui.confirm.delete" || key == "ui.confirm.remove")
}

fn read_into(dictionary: &mut Dictionary, path: &Path) {
    let texts: Dictionary = serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap();
    dictionary.extend(texts);
}

fn dictionary() -> Dictionary {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut dictionary = Dictionary::new();
    read_into(&mut dictionary, &root.join("app/ui/src/locales/en.json"));
    read_into(&mut dictionary, &root.join("crates/core/locales/en.json"));
    for module in fs::read_dir(root.join("modules")).unwrap() {
        let locale = module.unwrap().path().join("locales/en.json");
        if locale.is_file() {
            read_into(&mut dictionary, &locale);
        }
    }
    dictionary
}

/// The literal parts of a text around its `{placeholders}`.
fn literals(text: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut rest = text;
    while let Some((before, after)) = rest.split_once('{') {
        parts.push(before);
        rest = after.split_once('}').map_or("", |(_, tail)| tail);
    }
    parts.push(rest);
    parts
}

/// Whether `name` is `text` with its placeholders filled in.
fn fills(text: &str, name: &str) -> bool {
    let parts = literals(text);
    let (Some(first), Some(last)) = (parts.first(), parts.last()) else { return false };
    if parts.len() == 1 {
        return text == name;
    }
    if parts.iter().all(|part| part.is_empty()) {
        return false;
    }
    let Some(mut rest) = name.strip_prefix(first) else { return false };
    for part in &parts[1..parts.len() - 1] {
        let Some((_, after)) = rest.split_once(part) else { return false };
        rest = after;
    }
    rest.ends_with(last)
}

/// Whether `start` begins some filling of `text`: it must agree with the text up to its first placeholder.
fn begins(text: &str, start: &str) -> bool {
    let first = literals(text)[0];
    !first.is_empty() && (first.starts_with(start) || start.starts_with(first))
}

/// The key of the text a target's name is, or why there is none.
fn key_of(target: &Target, dictionary: &Dictionary) -> Result<String, String> {
    let keys: Vec<&String> = dictionary
        .iter()
        .filter(|(_, text)| match &target.name {
            NameMatch::Exact { text: name } => fills(text, name),
            NameMatch::Start { text: name } => begins(text, name),
        })
        .map(|(key, _)| key)
        .collect();
    let (NameMatch::Exact { text: name } | NameMatch::Start { text: name }) = &target.name;
    // The most specific text wins, but a permanent one is traced first so that it is refused.
    let specific = |key: &&&String| literals(&dictionary[**key]).iter().map(|part| part.len()).sum::<usize>();
    let permanent = keys.iter().find(|key| refused_key(key));
    permanent
        .or_else(|| keys.iter().rev().max_by_key(specific))
        .map(|key| (*key).clone())
        .ok_or_else(|| format!("{name:?} is no text of the interface"))
}

fn permanent(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.contains("permanent") || lower.contains("delete")
}

fn target_of(step: &Step) -> Option<&Target> {
    match step {
        Step::Click { target }
        | Step::Hover { target }
        | Step::Scroll { target, .. }
        | Step::Wait { until: Wait::Target { target } } => Some(target),
        Step::Wait { until: Wait::State { .. } } | Step::Pause { .. } => None,
    }
}

/// One line per target, `role "name" <- key`; an error names the first step that has no text or the permanent choice.
fn trace(scenario: &Scenario, dictionary: &Dictionary) -> Result<Vec<String>, String> {
    let mut lines = Vec::new();
    for (index, step) in scenario.steps.iter().enumerate() {
        let Some(target) = target_of(step) else { continue };
        let number = index + 1;
        let (NameMatch::Exact { text: name } | NameMatch::Start { text: name }) = &target.name;
        let key = key_of(target, dictionary).map_err(|reason| format!("step {number}: {reason}"))?;
        if permanent(name) || refused_key(&key) {
            return Err(format!("step {number}: {name:?} is the permanent delete; a demo only moves to the Trash"));
        }
        lines.push(format!("{:?} {name:?} <- {key}", target.role));
    }
    Ok(lines)
}

fn scenario(steps: &str) -> Scenario {
    let head = "world = \"developer-mac\"\nwindow = { width = 1100, height = 720 }\n";
    Scenario::parse("traced", &format!("{head}{steps}")).unwrap()
}

mod negative {
    use super::{dictionary, scenario, trace};

    #[test]
    fn a_name_no_text_says_is_refused_naming_its_step() {
        let steps = "[[step]]\npause = 100\n[[step]]\nclick = { role = \"button\", name = \"Frobnicate\" }\n";

        let error = trace(&scenario(steps), &dictionary()).unwrap_err();

        assert!(error.starts_with("step 2: \"Frobnicate\""), "{error}");
    }

    #[test]
    fn the_permanent_choice_is_refused_by_its_whole_text() {
        let steps = "[[step]]\nclick = { role = \"checkbox\", name = \"Delete permanently instead — this cannot be undone\" }\n";

        let error = trace(&scenario(steps), &dictionary()).unwrap_err();

        assert!(error.contains("permanent delete"), "{error}");
    }

    #[test]
    fn the_permanent_button_is_refused_by_its_text_or_its_start() {
        for target in ["name = \"Delete permanently\"", "name-start = \"Delete perm\""] {
            let steps = format!("[[step]]\nclick = {{ role = \"button\", {target} }}\n");

            let error = trace(&scenario(&steps), &dictionary()).unwrap_err();

            assert!(error.contains("permanent delete"), "{target}: {error}");
        }
    }

    #[test]
    fn the_tool_removal_button_is_refused() {
        let steps = "[[step]]\nclick = { role = \"button\", name = \"Remove\" }\n";

        let error = trace(&scenario(steps), &dictionary()).unwrap_err();

        assert!(error.contains("permanent delete"), "{error}");
    }

    #[test]
    fn a_hover_or_a_wait_on_the_permanent_choice_is_refused_too() {
        for step in [
            "hover = { role = \"button\", name = \"Delete permanently\" }",
            "wait = { role = \"button\", name = \"Delete permanently\" }",
        ] {
            let error = trace(&scenario(&format!("[[step]]\n{step}\n")), &dictionary()).unwrap_err();

            assert!(error.contains("permanent delete"), "{step}: {error}");
        }
    }
}

mod positive {
    use std::fs;
    use std::path::Path;

    use super::{Scenario, dictionary, scenario, trace};

    #[test]
    fn a_name_with_a_placeholder_is_traced_to_its_text() {
        let steps = "[[step]]\nclick = { role = \"checkbox\", name = \"Select Xcode build data\" }\n";

        let lines = trace(&scenario(steps), &dictionary()).unwrap();

        assert_eq!(lines, ["Checkbox \"Select Xcode build data\" <- ui.category-card.select"]);
    }

    #[test]
    fn a_start_of_a_name_is_traced_to_its_text() {
        let steps = "[[step]]\nclick = { role = \"button\", name-start = \"Cleanup of\" }\n";

        let lines = trace(&scenario(steps), &dictionary()).unwrap();

        assert_eq!(lines, ["Button \"Cleanup of\" <- ui.cleanup-row.title"]);
    }

    /// Every target of every scenario of the repository is a text of the interface and never the permanent delete.
    #[test]
    fn every_target_of_the_repository_scenarios_is_traced() {
        let dictionary = dictionary();
        let scenarios = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/demos/scenarios");
        for entry in fs::read_dir(scenarios).unwrap() {
            let path = entry.unwrap().path();
            let scenario = Scenario::read(&path).unwrap();
            let traced = trace(&scenario, &dictionary);

            assert!(traced.is_ok(), "{}: {:?}", path.display(), traced.err());
        }
    }
}
