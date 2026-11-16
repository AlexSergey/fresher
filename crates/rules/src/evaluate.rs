//! A match the walk delivered, judged against the rule that asked for it.

use fresher_core::{
    CACHEDIR_TAG, EntryKind, MarkerCheck, Match, ModuleContext, NewFinding, Pattern, Removal, Subject, Text, TextArg,
    Timestamp,
};

use crate::RuleSet;
use crate::format::RemovalSpec;
use crate::rules::Rule;

const SECONDS_PER_DAY: u64 = 24 * 60 * 60;

impl RuleSet {
    /// The finding for `matched` when the rule at `matched.interest` claims it; `None` otherwise. A symlink is never
    /// claimed; `must-not-match` is checked first and wins; then the name, the version, the age (the clock of `cx`
    /// against the match's modification time) and the sibling must all hold. A `marker = "CACHEDIR.TAG"` rule also
    /// needs the walk's check of the tag's signature. Pure: no IO.
    pub fn evaluate(&self, matched: &Match, cx: &ModuleContext<'_>) -> Option<NewFinding> {
        let rule = self.rules.get(matched.interest)?;
        let name = matched.path.file_name()?.to_str()?;
        let now = Timestamp::from_system_time(cx.clock().now());
        rule.claims(matched, name, now).then(|| rule.finding(matched, name))
    }
}

impl Rule {
    fn claims(&self, matched: &Match, name: &str, now: Timestamp) -> bool {
        if matched.kind == EntryKind::Symlink || self.must_not_match.iter().any(|glob| glob.matches_ignoring_case(name))
        {
            return false;
        }
        let folder_only =
            self.removal == RemovalSpec::EmptyFolder || matches!(self.interest.pattern(), Pattern::MarkerFile(_));
        if folder_only && matched.kind != EntryKind::Folder {
            return false;
        }
        self.name.as_ref().is_none_or(|glob| glob.matches(name))
            && self.version.as_ref().is_none_or(|pattern| pattern.find(name).is_some())
            && self.min_age_days.is_none_or(|days| old_enough(matched.modified, now, days))
            && self.sibling.as_ref().is_none_or(|sibling| matched.siblings.contains(sibling))
            && self.tag_holds(matched)
    }

    /// A `CACHEDIR.TAG` marker counts only when the walk found its signature; other markers need nothing more.
    fn tag_holds(&self, matched: &Match) -> bool {
        match self.interest.pattern() {
            Pattern::MarkerFile(marker) if marker == CACHEDIR_TAG => matched.marker == Some(MarkerCheck::Valid),
            _ => true,
        }
    }

    fn finding(&self, matched: &Match, name: &str) -> NewFinding {
        let path = matched.path.clone();
        let removal = match self.removal {
            RemovalSpec::Trash => Removal::MoveToTrash(path.clone()),
            RemovalSpec::EmptyFolder => Removal::EmptyFolder(path.clone()),
        };
        let mut finding = NewFinding::new(
            &self.category,
            path.to_string_lossy().into_owned(),
            Subject::Path(path),
            matched.sizes,
            Text::new(self.title.clone()).with("name", TextArg::Text(name.into())),
            Text::new(self.reason.clone()),
            removal,
        );
        finding.raise_danger(self.danger);
        finding
    }
}

/// `true` when the item was last modified at least `days` before `now`; an unknown or future time is not old.
fn old_enough(modified: Option<Timestamp>, now: Timestamp, days: u32) -> bool {
    modified.is_some_and(|modified| now.secs().saturating_sub(modified.secs()) >= u64::from(days) * SECONDS_PER_DAY)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use fresher_core::{
        ByteSize, CancelToken, EntryKind, MarkerCheck, Match, ModuleContext, ModuleId, ModuleSettings, NewFinding,
        Sizes, Timestamp,
    };
    use fresher_testing::{FakeClock, FakeCommandRunner};

    use crate::RuleSet;
    use crate::rules::sample;

    /// The clock of every test: day 100 after the epoch.
    const NOW: u64 = 100 * 86_400;

    /// `caches` (`Library/Caches/*`, `com.example.*`, 30 days, `Info.plist` next to it, danger medium over the Low
    /// category, emptied) and `tagged` (any folder with a `CACHEDIR.TAG`).
    fn rules() -> RuleSet {
        let toml = r#"
[[rule]]
id = "caches"
category = "example.caches"
danger = "medium"
removal = "empty-folder"
[rule.look]
under = { folder = "home", path = "Library/Caches/*" }
[rule.recognize]
name = "com.example.*"
min-age-days = 30
sibling = "Info.plist"
[[rule.must-not-match]]
name = "com.example.keep*"

[[rule]]
id = "tagged"
category = "example.caches"
danger = "low"
removal = "trash"
[rule.look]
marker = "CACHEDIR.TAG"
[[rule.must-not-match]]
name = "Library"
"#;
        RuleSet::parse(&sample::module(), toml).unwrap()
    }

    /// A match of the `caches` rule that it claims: old enough, with its sibling.
    fn cache(name: &str) -> Match {
        Match {
            interest: 0,
            path: Path::new("/home/Library/Caches").join(name),
            kind: EntryKind::Folder,
            sizes: Sizes::complete(ByteSize::new(10), ByteSize::new(4096)),
            modified: Some(Timestamp::from_secs(NOW - 31 * 86_400)),
            siblings: vec!["Info.plist".into()],
            marker: None,
            facts: fresher_core::EntryFacts::REMOVABLE,
        }
    }

    /// A match of the `tagged` rule, with the walk's check of the tag.
    fn tagged(marker: Option<MarkerCheck>) -> Match {
        Match { interest: 1, path: "/home/project/cache".into(), siblings: vec![], marker, ..cache("unused") }
    }

    fn evaluate(rules: &RuleSet, matched: &Match) -> Option<NewFinding> {
        let module = ModuleId::new("example").unwrap();
        let (clock, commands, cancel) = (FakeClock::at(NOW), FakeCommandRunner::default(), CancelToken::new());
        let settings = ModuleSettings::empty(module);
        let system = fresher_core::SystemFacts::new(&fresher_testing::fake_system(Path::new("/home")));
        rules.evaluate(matched, &ModuleContext::new(&clock, &commands, &system, &settings, &cancel))
    }

    mod negative {
        use fresher_core::{EntryKind, MarkerCheck, Match, Timestamp};

        use super::{NOW, cache, evaluate, rules, tagged};

        #[test]
        fn a_look_alike_name_is_not_claimed() {
            for name in ["com.examples.app", "org.example.app", "com.example", "Com.Example.app"] {
                assert_eq!(evaluate(&rules(), &cache(name)), None, "{name}");
            }
        }

        #[test]
        fn must_not_match_wins_over_a_matching_name() {
            for name in ["com.example.keeper", "COM.EXAMPLE.KEEP"] {
                assert_eq!(evaluate(&rules(), &cache(name)), None, "{name}");
            }
        }

        #[test]
        fn a_symlink_is_never_claimed() {
            let link = Match { kind: EntryKind::Symlink, ..cache("com.example.app") };

            assert_eq!(evaluate(&rules(), &link), None);
        }

        #[test]
        fn an_item_younger_than_min_age_is_not_claimed() {
            for modified in [Some(NOW - 29 * 86_400), Some(NOW + 86_400), None] {
                let young = Match { modified: modified.map(Timestamp::from_secs), ..cache("com.example.app") };

                assert_eq!(evaluate(&rules(), &young), None, "{modified:?}");
            }
        }

        #[test]
        fn an_item_without_its_sibling_or_of_the_wrong_kind_is_not_claimed() {
            let alone = Match { siblings: vec![], ..cache("com.example.app") };
            let file = Match { kind: EntryKind::File, ..cache("com.example.app") };
            let unknown_rule = Match { interest: 7, ..cache("com.example.app") };

            assert_eq!(evaluate(&rules(), &alone), None);
            assert_eq!(evaluate(&rules(), &file), None);
            assert_eq!(evaluate(&rules(), &unknown_rule), None);
            assert_eq!(evaluate(&rules(), &Match { path: "/".into(), ..cache("") }), None);
        }

        #[test]
        fn a_marker_rule_does_not_claim_an_invalid_tag() {
            assert_eq!(evaluate(&rules(), &tagged(Some(MarkerCheck::Invalid))), None);
        }

        #[test]
        fn a_marker_rule_does_not_claim_a_match_with_no_marker_check() {
            assert_eq!(evaluate(&rules(), &tagged(None)), None);
        }
    }

    mod positive {
        use std::path::PathBuf;

        use fresher_core::{DangerLevel, MarkerCheck, Removal, Subject, TextArg};

        use super::{cache, evaluate, rules, tagged};

        #[test]
        fn claims_a_matching_folder_with_the_rule_danger() {
            let matched = cache("com.example.app");

            let finding = evaluate(&rules(), &matched).unwrap();

            let path = PathBuf::from("/home/Library/Caches/com.example.app");
            assert_eq!(finding.danger(), DangerLevel::Medium);
            assert_eq!(finding.category().as_str(), "example.caches");
            assert_eq!(finding.key().identity(), "/home/Library/Caches/com.example.app");
            assert_eq!(finding.subject(), &Subject::Path(path.clone()));
            assert_eq!(finding.sizes(), matched.sizes);
            assert_eq!(finding.removal(), &Removal::EmptyFolder(path));
            assert_eq!(finding.title().key().as_str(), "example.rule.caches.title");
            assert_eq!(finding.title().args().get("name"), Some(&TextArg::Text("com.example.app".into())));
            assert_eq!(finding.reason().key().as_str(), "example.rule.caches.reason");
        }

        #[test]
        fn a_marker_rule_claims_a_valid_tag() {
            let matched = tagged(Some(MarkerCheck::Valid));

            let finding = evaluate(&rules(), &matched).unwrap();

            assert_eq!(finding.danger(), DangerLevel::Low);
            assert_eq!(finding.removal(), &Removal::MoveToTrash(matched.path));
            assert_eq!(finding.reason().key().as_str(), "example.rule.tagged.reason");
        }
    }
}
