//! The user's confirmation, and the only way from a plan to a removal.
//!
//! A [`Confirmation`] has no constructor: it exists only as the JSON the UI sends. [`CleanupPlan::confirm`] is the
//! only constructor of a [`ConfirmedPlan`], which is not `Clone` and is executed by value, once. Every removal
//! function of the cleaner takes an [`Act`], which only a confirmed plan lends, and a permanent delete also takes the
//! [`PermanentConsent`] only a permanent confirmation carries. A plan whose highest level is High is confirmed only
//! with the user's tick that its risk is understood, sent with the echo and never remembered. `tests/compile_fail/`
//! holds all of this.

use std::fmt;

use serde::Deserialize;
use ts_rs::TS;

use crate::cleanup::{CleanupPlan, Method, PLAN_LIFETIME, PlanId, PlanItem};
use crate::{ByteSize, DangerLevel, Timestamp};

/// What the confirmation showed the user, sent back by the UI with the plan's id: the count, the allocated bytes of
/// the total, the highest danger level, the method the user chose, and `highRiskUnderstood`, the user's tick that a
/// High item may be the only copy (required; it matters only for a plan whose highest level is High). Only
/// deserialized, never built in code.
#[derive(Debug, PartialEq, Eq, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct Confirmation {
    plan: PlanId,
    count: u32,
    allocated: ByteSize,
    highest: DangerLevel,
    method: Method,
    #[serde(rename = "highRiskUnderstood")]
    high_risk_understood: bool,
}

impl Confirmation {
    /// The plan the user confirmed.
    pub const fn plan(&self) -> PlanId {
        self.plan
    }

    pub const fn method(&self) -> Method {
        self.method
    }
}

/// A field of the confirmation that differs from the plan.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EchoField {
    Count,
    Allocated,
    Highest,
}

impl fmt::Display for EchoField {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Count => "count",
            Self::Allocated => "allocated size",
            Self::Highest => "highest danger level",
        })
    }
}

/// Why a confirmation was refused. The plan is consumed either way: the user prepares again.
#[derive(Clone, Copy, Debug, thiserror::Error, PartialEq, Eq)]
pub enum ConfirmError {
    /// The confirmation names another plan.
    #[error("the confirmation is of another plan")]
    WrongPlan,
    /// The plan is older than its lifetime, or was made after the time of the confirmation.
    #[error("the plan is older than its lifetime")]
    Expired,
    /// The UI showed something other than the plan.
    #[error("the confirmed {field} differs from the plan's")]
    Mismatch { field: EchoField },
    /// The plan holds a High item and the user did not tick that its risk is understood.
    #[error("the plan holds a High item and its risk was not confirmed")]
    HighRiskNotUnderstood,
}

/// A plan the user confirmed: the only input of the cleaner's removing entry point. Not `Clone`, never
/// (de)serialized, built only by [`CleanupPlan::confirm`].
#[derive(Debug)]
pub struct ConfirmedPlan {
    plan: CleanupPlan,
    method: ConfirmedMethod,
}

/// The method of a confirmed plan; the permanent one carries its consent.
#[derive(Debug)]
pub enum ConfirmedMethod {
    Trash,
    Permanent(PermanentConsent),
}

/// The user's consent to delete permanently: made only by [`CleanupPlan::confirm`] for `Method::Permanent`, and asked
/// for by the cleaner's permanent removal. A module, a setting or a default can never produce one.
#[derive(Debug)]
pub struct PermanentConsent(());

/// The licence to act on the items of one confirmed plan, borrowed from it: every removal function of the cleaner
/// takes one, so nothing can be removed outside a confirmed plan.
#[derive(Clone, Copy, Debug)]
pub struct Act<'a> {
    plan: &'a ConfirmedPlan,
}

impl CleanupPlan {
    /// The only way to a [`ConfirmedPlan`]: the confirmation must name this plan, the plan must be at most
    /// [`PLAN_LIFETIME`] old at `now`, and the count, the allocated bytes and the highest danger level the user saw
    /// must equal the plan's; a plan whose highest level is High also needs the user's tick that its risk is
    /// understood. The plan is consumed whatever the answer.
    pub fn confirm(self, echo: &Confirmation, now: Timestamp) -> Result<ConfirmedPlan, ConfirmError> {
        if echo.plan != self.id() {
            return Err(ConfirmError::WrongPlan);
        }
        let age = now.secs().checked_sub(self.made().secs()).ok_or(ConfirmError::Expired)?;
        if age > PLAN_LIFETIME.as_secs() {
            return Err(ConfirmError::Expired);
        }
        let summary = self.summary();
        let mismatch = |field| Err(ConfirmError::Mismatch { field });
        if echo.count != summary.count {
            return mismatch(EchoField::Count);
        }
        if echo.allocated != summary.sizes.allocated {
            return mismatch(EchoField::Allocated);
        }
        if echo.highest != summary.highest {
            return mismatch(EchoField::Highest);
        }
        if summary.highest == DangerLevel::High && !echo.high_risk_understood {
            return Err(ConfirmError::HighRiskNotUnderstood);
        }
        let method = match echo.method {
            Method::Trash => ConfirmedMethod::Trash,
            Method::Permanent => ConfirmedMethod::Permanent(PermanentConsent(())),
        };
        Ok(ConfirmedPlan { plan: self, method })
    }
}

impl ConfirmedPlan {
    /// The plan as the user confirmed it.
    pub const fn plan(&self) -> &CleanupPlan {
        &self.plan
    }

    /// The items, in the order the cleaner acts on them.
    pub fn items(&self) -> &[PlanItem] {
        self.plan.items()
    }

    pub const fn method(&self) -> Method {
        match self.method {
            ConfirmedMethod::Trash => Method::Trash,
            ConfirmedMethod::Permanent(_) => Method::Permanent,
        }
    }

    /// The consent to delete permanently; `None` for a Trash plan.
    pub const fn permanent(&self) -> Option<&PermanentConsent> {
        match &self.method {
            ConfirmedMethod::Trash => None,
            ConfirmedMethod::Permanent(consent) => Some(consent),
        }
    }

    /// The licence to act on this plan's items.
    pub const fn act(&self) -> Act<'_> {
        Act { plan: self }
    }
}

impl<'a> Act<'a> {
    /// The confirmed plan this licence was lent by.
    pub const fn plan(self) -> &'a ConfirmedPlan {
        self.plan
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::cleanup::Confirmation;
    use crate::cleanup::plan::sample;

    /// The confirmation of the sample plan as the UI would send it, with `change` applied to its JSON.
    fn echo(change: impl FnOnce(&mut serde_json::Value)) -> Confirmation {
        let mut json = json!({
            "plan": 7, "count": 3, "allocated": 4096 + 8192 + 100, "highest": "medium", "method": "trash",
            "highRiskUnderstood": false,
        });
        change(&mut json);
        serde_json::from_value(json).unwrap()
    }

    /// The confirmation of the sample High plan, the risk understood as given.
    fn high_echo(understood: bool, change: impl FnOnce(&mut serde_json::Value)) -> Confirmation {
        echo(|json| {
            json["count"] = 2.into();
            json["allocated"] = (4096 + 16_384).into();
            json["highest"] = "high".into();
            json["highRiskUnderstood"] = understood.into();
            change(json);
        })
    }

    /// One second past the sample plan's lifetime.
    fn too_late() -> crate::Timestamp {
        crate::Timestamp::from_secs(sample::MADE + crate::cleanup::PLAN_LIFETIME.as_secs() + 1)
    }

    mod negative {
        use super::{echo, high_echo, too_late};
        use crate::Timestamp;
        use crate::cleanup::plan::sample;
        use crate::cleanup::{ConfirmError, Confirmation, EchoField};

        fn now() -> Timestamp {
            Timestamp::from_secs(sample::MADE + 60)
        }

        #[test]
        fn a_confirmation_of_another_plan_is_refused() {
            let error = sample::plan().confirm(&echo(|json| json["plan"] = 8.into()), now());

            assert_eq!(error.unwrap_err(), ConfirmError::WrongPlan);
        }

        #[test]
        fn a_different_count_is_refused() {
            let error = sample::plan().confirm(&echo(|json| json["count"] = 300.into()), now());

            assert_eq!(error.unwrap_err(), ConfirmError::Mismatch { field: EchoField::Count });
        }

        #[test]
        fn a_different_size_is_refused() {
            let error = sample::plan().confirm(&echo(|json| json["allocated"] = 1.into()), now());

            assert_eq!(error.unwrap_err(), ConfirmError::Mismatch { field: EchoField::Allocated });
        }

        #[test]
        fn a_different_highest_level_is_refused() {
            let error = sample::plan().confirm(&echo(|json| json["highest"] = "low".into()), now());

            assert_eq!(error.unwrap_err(), ConfirmError::Mismatch { field: EchoField::Highest });
        }

        #[test]
        fn a_plan_older_than_its_lifetime_is_refused() {
            assert_eq!(sample::plan().confirm(&echo(|_| ()), too_late()).unwrap_err(), ConfirmError::Expired);
        }

        #[test]
        fn a_confirmation_from_before_the_plan_is_refused() {
            let before = Timestamp::from_secs(sample::MADE - 1);

            assert_eq!(sample::plan().confirm(&echo(|_| ()), before).unwrap_err(), ConfirmError::Expired);
        }

        #[test]
        fn a_permanent_method_alone_gives_no_consent_for_the_trash() {
            let confirmed = sample::plan().confirm(&echo(|_| ()), now()).unwrap();

            assert!(confirmed.permanent().is_none());
        }

        #[test]
        fn a_confirmation_with_an_unknown_field_is_refused() {
            let mut json = serde_json::json!({
                "plan": 7, "count": 3, "allocated": 1, "highest": "low", "method": "trash", "highRiskUnderstood": false,
            });
            json["items"] = serde_json::json!([1, 2, 3]);

            assert!(serde_json::from_value::<Confirmation>(json).is_err());
        }

        #[test]
        fn a_high_plan_without_the_risk_understood_is_refused() {
            let error = sample::high_plan().confirm(&high_echo(false, |_| ()), now());

            assert_eq!(error.unwrap_err(), ConfirmError::HighRiskNotUnderstood);
        }

        #[test]
        fn a_high_plan_with_a_wrong_count_is_refused_for_the_count_not_the_risk() {
            let error = sample::high_plan().confirm(&high_echo(false, |json| json["count"] = 5.into()), now());

            assert_eq!(error.unwrap_err(), ConfirmError::Mismatch { field: EchoField::Count });
        }

        #[test]
        fn the_refusal_consumes_the_plan() {
            let plan = sample::high_plan();

            let refused = plan.confirm(&high_echo(false, |_| ()), now());

            // `plan` was moved into `confirm`: a second confirmation of it does not compile, so the user prepares again.
            assert!(refused.is_err());
        }

        #[test]
        fn a_confirmation_without_the_field_is_not_read() {
            let json =
                serde_json::json!({ "plan": 7, "count": 3, "allocated": 1, "highest": "low", "method": "trash" });

            assert!(serde_json::from_value::<Confirmation>(json).is_err());
        }

        #[test]
        fn errors_name_what_differs() {
            assert_eq!(
                ConfirmError::Mismatch { field: EchoField::Allocated }.to_string(),
                "the confirmed allocated size differs from the plan's"
            );
            assert_eq!(EchoField::Count.to_string(), "count");
            assert_eq!(EchoField::Highest.to_string(), "highest danger level");
            assert_ne!(ConfirmError::WrongPlan.to_string(), ConfirmError::Expired.to_string());
            assert_eq!(
                ConfirmError::HighRiskNotUnderstood.to_string(),
                "the plan holds a High item and its risk was not confirmed"
            );
        }
    }

    mod positive {
        use super::{echo, high_echo};
        use crate::Timestamp;
        use crate::cleanup::plan::sample;
        use crate::cleanup::{Method, PLAN_LIFETIME, PlanId};

        #[test]
        fn a_matching_confirmation_confirms() {
            let echo = echo(|_| ());
            let at_the_last_second = Timestamp::from_secs(sample::MADE + PLAN_LIFETIME.as_secs());

            let confirmed = sample::plan().confirm(&echo, at_the_last_second).unwrap();

            assert_eq!(echo.plan(), PlanId::new(7));
            assert_eq!(echo.method(), Method::Trash);
            assert_eq!(confirmed.method(), Method::Trash);
            assert_eq!(confirmed.items().len(), 3);
            assert_eq!(confirmed.plan().id(), PlanId::new(7));
            let act = confirmed.act();
            assert!(std::ptr::eq(act.plan(), &raw const confirmed));
        }

        #[test]
        fn a_permanent_confirmation_carries_the_consent() {
            let confirmed = sample::plan()
                .confirm(&echo(|json| json["method"] = "permanent".into()), Timestamp::from_secs(sample::MADE))
                .unwrap();

            assert_eq!(confirmed.method(), Method::Permanent);
            assert!(confirmed.permanent().is_some());
        }

        #[test]
        fn a_high_plan_with_the_risk_understood_is_confirmed() {
            let confirmed = sample::high_plan().confirm(&high_echo(true, |_| ()), Timestamp::from_secs(sample::MADE));

            assert_eq!(confirmed.unwrap().items().len(), 2);
        }

        #[test]
        fn a_plan_below_high_is_confirmed_with_the_field_false() {
            let confirmed = sample::plan().confirm(&echo(|_| ()), Timestamp::from_secs(sample::MADE));

            assert_eq!(confirmed.unwrap().method(), Method::Trash);
        }

        #[test]
        fn the_field_true_on_a_low_plan_changes_nothing() {
            let understood = echo(|json| json["highRiskUnderstood"] = true.into());

            let confirmed = sample::plan().confirm(&understood, Timestamp::from_secs(sample::MADE)).unwrap();

            assert_eq!(confirmed.method(), Method::Trash);
            assert!(confirmed.permanent().is_none());
            assert_eq!(confirmed.items().len(), 3);
        }

        #[test]
        fn the_lifetime_is_ten_minutes() {
            assert_eq!(PLAN_LIFETIME.as_secs(), 600);
        }
    }

    mod property {
        use proptest::prelude::*;
        use serde_json::json;

        use crate::Timestamp;
        use crate::cleanup::plan::sample;
        use crate::cleanup::{ConfirmError, Confirmation, Method};

        fn level() -> impl Strategy<Value = &'static str> {
            prop_oneof![Just("safe"), Just("low"), Just("medium"), Just("high")]
        }

        proptest! {
            /// Any change of the plan, the count, the size or the level refuses; the method is the user's choice and
            /// is carried as it was sent.
            #[test]
            fn any_changed_field_of_the_echo_refuses(
                plan in prop_oneof![Just(7_u32), any::<u32>()],
                count in prop_oneof![Just(3_u32), any::<u32>()],
                allocated in prop_oneof![Just(12_388_u64), 0..=u64::MAX >> 11],
                highest in prop_oneof![Just("medium"), level()],
                permanent: bool,
                understood: bool,
            ) {
                let method = if permanent { "permanent" } else { "trash" };
                let json = json!({
                    "plan": plan, "count": count, "allocated": allocated, "highest": highest, "method": method,
                    "highRiskUnderstood": understood,
                });
                let echo: Confirmation = serde_json::from_value(json).unwrap();
                let same = plan == 7 && count == 3 && allocated == 12_388 && highest == "medium";

                let answer = sample::plan().confirm(&echo, Timestamp::from_secs(sample::MADE + 1));

                match answer {
                    Ok(confirmed) => {
                        prop_assert!(same);
                        let expected = if permanent { Method::Permanent } else { Method::Trash };
                        prop_assert_eq!(confirmed.method(), expected);
                        prop_assert_eq!(confirmed.permanent().is_some(), permanent);
                    }
                    Err(error) => {
                        prop_assert!(!same, "{error}");
                        let refused = matches!(error, ConfirmError::WrongPlan | ConfirmError::Mismatch { .. });
                        prop_assert!(refused, "{}", error);
                    }
                }
            }
        }
    }
}
