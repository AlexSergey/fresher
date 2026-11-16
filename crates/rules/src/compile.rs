//! A rule set compiled to the interests the walk looks for.

use fresher_core::Interest;

use crate::RuleSet;

impl RuleSet {
    /// What the module asks the walk for: one interest per rule, in the order of the rules, so that the interest at
    /// `index` serves `rules()[index]` and [`RuleSet::evaluate`] finds the rule by `Match::interest`.
    pub fn interests(&self) -> Vec<Interest> {
        self.rules.iter().map(|rule| rule.interest.clone()).collect()
    }
}

#[cfg(test)]
mod tests {
    mod negative {
        use crate::RuleSet;
        use crate::rules::sample;

        #[test]
        fn no_rules_ask_for_nothing() {
            assert_eq!(RuleSet::parse(&sample::module(), "").unwrap().interests(), []);
        }
    }

    mod positive {
        use fresher_core::{Claim, KnownFolder, Pattern};

        use crate::RuleSet;
        use crate::rules::sample;

        #[test]
        fn each_rule_serves_the_interest_at_its_index() {
            let rules = RuleSet::parse(&sample::module(), sample::COMPLETE).unwrap();

            let interests = rules.interests();

            assert_eq!(interests.len(), rules.rules().len());
            assert_eq!(interests[0].pattern(), &Pattern::Under(KnownFolder::Home, "Library/Caches/*".into()));
            assert_eq!(interests[0].claim(), Claim::Whole);
            assert_eq!(interests[0].siblings(), ["package.json"]);
        }
    }
}
