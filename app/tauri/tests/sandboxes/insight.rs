//! The insight check of a case file: every insight the case file's module described is expected by one of its cases'
//! `[[case.insight]]`, every expected one is described, and its figures, parts and commands are counted as expected.

use std::path::Path;

use fresher_core::{Insight, ModuleId};
use fresher_sandbox::CaseFile;
use serde_json::Value;

use crate::harness::{Failure, failure};

/// The insights a scan's modules described, each with its module, in the order of their `ModuleFinished` events.
pub(crate) type Described = Vec<(ModuleId, Insight)>;

/// Fails on the first insight of the file's module (of any module when the file names none) that no case expects, on
/// the first expected insight it did not describe, and on the first one whose counts differ, naming every count.
pub(crate) fn check_insights(name: &str, file: &CaseFile, described: &[(ModuleId, Insight)]) -> Result<(), Failure> {
    let own: Vec<&(ModuleId, Insight)> =
        described.iter().filter(|(module, _)| file.module().is_none_or(|own| own == *module)).collect();
    let expected = file.cases().iter().flat_map(|case| case.insights().iter().map(move |insight| (case, insight)));
    let expected: Vec<_> = expected.collect();
    for (module, insight) in &own {
        if !expected.iter().any(|(_, expect)| expect.identity() == insight.identity()) {
            let problem = format!(
                "insight `{}` of `{}` is described but no case expects it",
                insight.identity(),
                module.as_str()
            );
            return Err(failure(name, problem));
        }
    }
    let module = file.module().map_or("the modules", ModuleId::as_str);
    for (case, expect) in expected {
        let Some((_, insight)) = own.iter().find(|(_, insight)| insight.identity() == expect.identity()) else {
            let (case_name, identity) = (case.name(), expect.identity());
            let problem = format!("case `{case_name}` expects insight `{identity}`, which `{module}` did not describe");
            return Err(failure(case.name(), problem));
        };
        let commands = insight.parts().iter().filter(|part| part.command.is_some()).count();
        let counts = [
            ("figures", expect.figures(), insight.figures().len()),
            ("parts", expect.parts(), insight.parts().len()),
            ("commands", expect.commands(), commands),
        ];
        let differ: Vec<String> = counts
            .into_iter()
            .filter(|&(_, want, got)| usize::try_from(want).ok() != Some(got))
            .map(|(what, want, got)| format!("{what}: expected {want}, described {got}"))
            .collect();
        if !differ.is_empty() {
            let problem = format!("insight `{}` of `{module}`: {}", expect.identity(), differ.join("; "));
            return Err(failure(case.name(), problem));
        }
    }
    Ok(())
}

/// The report's `insights`: the file's module's insights as serialized, in order, every path relative to the sandbox;
/// `None` when there are none, so a case file without insights keeps the report it always had.
pub(crate) fn report(file: &CaseFile, described: &[(ModuleId, Insight)], sandbox: &Path) -> Option<Value> {
    let insights: Vec<Value> = described
        .iter()
        .filter(|(module, _)| file.module().is_none_or(|own| own == *module))
        .map(|(_, insight)| relative_paths(serde_json::to_value(insight).unwrap(), sandbox))
        .collect();
    (!insights.is_empty()).then_some(Value::Array(insights))
}

/// `value` with every string that is a path below `sandbox` made relative to it.
fn relative_paths(value: Value, sandbox: &Path) -> Value {
    match value {
        Value::String(text) => match Path::new(&text).strip_prefix(sandbox) {
            Ok(inside) => Value::String(inside.to_string_lossy().into_owned()),
            Err(_) => Value::String(text),
        },
        Value::Array(items) => Value::Array(items.into_iter().map(|item| relative_paths(item, sandbox)).collect()),
        Value::Object(fields) => {
            Value::Object(fields.into_iter().map(|(key, field)| (key, relative_paths(field, sandbox))).collect())
        }
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use fresher_core::{
        ByteSize, DangerLevel, Figure, Insight, InsightPart, ModuleId, ShownCommand, Text, TextKey, ToolId,
    };
    use fresher_sandbox::CaseFile;

    fn text(key: &str) -> Text {
        Text::new(TextKey::new(key).unwrap())
    }

    fn docker() -> ModuleId {
        ModuleId::new("docker").unwrap()
    }

    /// An insight called `identity` with `figures` figures and `parts` parts, the first `commands` with a command.
    fn insight(identity: &str, figures: usize, parts: usize, commands: usize) -> Insight {
        let figure = Figure { label: text("docker.figure"), size: ByteSize::new(4096) };
        let part = |index: usize| InsightPart {
            label: text("docker.part.label"),
            size: ByteSize::new(2048),
            detail: text("docker.part.detail"),
            effect: text("docker.part.effect"),
            danger: DangerLevel::Safe,
            command: (index < commands).then(|| {
                let args = ["volume", "prune"].map(str::to_owned).to_vec();
                ShownCommand::new(ToolId::new("docker").unwrap(), args).unwrap()
            }),
        };
        let figures = vec![figure; figures];
        Insight::new(
            identity,
            text("docker.title"),
            text("docker.summary"),
            figures,
            (0..parts).map(part).collect(),
            vec![],
        )
        .unwrap()
    }

    /// A `docker` case file whose one negative case expects `insights` (`[[case.insight]]` tables).
    fn file(insights: &str) -> CaseFile {
        let text = format!(
            "feature = \"docker\"\nmodule = \"docker\"\n[[case]]\nname = \"orbstack answers\"\nkind = \"negative\"\n\
             [[case.file]]\npath = \"home/a.bin\"\n{insights}"
        );
        CaseFile::parse(&text).unwrap()
    }

    fn expects(identity: &str, figures: u32, parts: u32, commands: u32) -> String {
        format!(
            "[[case.insight]]\nidentity = \"{identity}\"\nfigures = {figures}\nparts = {parts}\ncommands = {commands}\n"
        )
    }

    mod negative {
        use super::{docker, expects, file, insight};
        use crate::insight::check_insights;

        #[test]
        fn an_insight_no_case_expects_fails_naming_it() {
            let failure = check_insights("docker", &file(""), &[(docker(), insight("orbstack", 3, 4, 3))]).unwrap_err();

            assert_eq!(failure.case, "docker");
            assert_eq!(failure.problem, "insight `orbstack` of `docker` is described but no case expects it");
        }

        #[test]
        fn an_expected_insight_not_described_fails() {
            let file = file(&expects("orbstack", 3, 4, 3));

            let failure = check_insights("docker", &file, &[]).unwrap_err();

            assert_eq!(failure.case, "orbstack answers");
            assert_eq!(
                failure.problem,
                "case `orbstack answers` expects insight `orbstack`, which `docker` did not describe"
            );
        }

        #[test]
        fn an_insight_of_another_module_is_not_the_files() {
            let other = fresher_core::ModuleId::new("trial").unwrap();

            let failure = check_insights(
                "docker",
                &file(&expects("orbstack", 3, 4, 3)),
                &[(other, insight("orbstack", 3, 4, 3))],
            )
            .unwrap_err();

            assert!(failure.problem.ends_with("which `docker` did not describe"), "{failure}");
        }

        #[test]
        fn an_insight_count_that_differs_fails_with_both_numbers() {
            let file = file(&expects("orbstack", 2, 4, 3));

            let failure = check_insights("docker", &file, &[(docker(), insight("orbstack", 3, 4, 2))]).unwrap_err();

            assert_eq!(failure.case, "orbstack answers");
            assert_eq!(
                failure.problem,
                "insight `orbstack` of `docker`: figures: expected 2, described 3; commands: expected 3, described 2"
            );
        }
    }

    mod positive {
        use std::path::Path;

        use fresher_core::{Text, TextArg, TextKey};
        use serde_json::json;

        use super::{docker, expects, file, insight};
        use crate::insight::{check_insights, report};

        #[test]
        fn an_expected_insight_passes_and_is_reported() {
            let file = file(&format!("{}{}", expects("orbstack", 3, 4, 3), expects("docker-desktop", 1, 0, 0)));
            let mut answered = insight("orbstack", 3, 4, 3);
            let at =
                Text::new(TextKey::new("docker.disk").unwrap()).with("path", TextArg::Path("/box/home/d.img".into()));
            answered = fresher_core::Insight::new(
                answered.identity(),
                at,
                answered.summary().clone(),
                answered.figures().to_vec(),
                answered.parts().to_vec(),
                vec![],
            )
            .unwrap();
            let described = [(docker(), answered), (docker(), insight("docker-desktop", 1, 0, 0))];

            check_insights("docker", &file, &described).unwrap();
            let report = report(&file, &described, Path::new("/box")).unwrap();

            let identities: Vec<&str> =
                report.as_array().unwrap().iter().map(|insight| insight["identity"].as_str().unwrap()).collect();
            assert_eq!(identities, ["orbstack", "docker-desktop"]);
            assert_eq!(report[0]["title"]["args"]["path"], json!({ "kind": "path", "value": "home/d.img" }));
            assert_eq!(report[0]["parts"].as_array().unwrap().len(), 4);
        }

        #[test]
        fn no_insight_reports_nothing() {
            let file = file("");

            check_insights("docker", &file, &[]).unwrap();
            assert_eq!(report(&file, &[], Path::new("/box")), None);
        }
    }
}
