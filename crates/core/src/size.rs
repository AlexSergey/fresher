use std::fmt;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// A number of bytes on disk; a `number` in TypeScript (exact below 2^53 bytes, 9 PB).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, TS)]
pub struct ByteSize(#[ts(type = "number")] u64);

impl ByteSize {
    pub const ZERO: Self = Self(0);

    pub const fn new(bytes: u64) -> Self {
        Self(bytes)
    }

    pub const fn bytes(self) -> u64 {
        self.0
    }

    /// Adds two sizes; `None` when the sum does not fit, so an overflow is never silent.
    pub const fn checked_add(self, other: Self) -> Option<Self> {
        match self.0.checked_add(other.0) {
            Some(sum) => Some(Self(sum)),
            None => None,
        }
    }
}

impl fmt::Display for ByteSize {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} B", self.0)
    }
}

/// How much of a size is known: `Complete` ("42 GB"), `Partial` ("42 GB+": some of it could not be read) or `Unknown`
/// ("?"). Ordered from the strongest to the weakest.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "kebab-case")]
pub enum Completeness {
    #[default]
    Complete,
    Partial,
    Unknown,
}

/// The size of an item: the bytes its files hold, the bytes they take on disk, the bytes removing it frees, and how
/// much of that is known.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
pub struct Sizes {
    pub logical: ByteSize,
    pub allocated: ByteSize,
    /// The private part of `allocated`: not shared with a clone or another hard link.
    pub reclaimable: ByteSize,
    pub completeness: Completeness,
}

impl Sizes {
    /// Nothing, completely known.
    pub const ZERO: Self = Self::complete(ByteSize::ZERO, ByteSize::ZERO);

    /// A completely known size whose bytes are all private: removing it frees all it takes.
    pub const fn complete(logical: ByteSize, allocated: ByteSize) -> Self {
        Self::new(logical, allocated, allocated)
    }

    /// A completely known size with its private part told apart.
    pub const fn new(logical: ByteSize, allocated: ByteSize, reclaimable: ByteSize) -> Self {
        Self { logical, allocated, reclaimable, completeness: Completeness::Complete }
    }

    /// Adds two sizes, keeping the weaker completeness; `None` when a sum does not fit.
    pub fn checked_add(self, other: Self) -> Option<Self> {
        Some(Self {
            logical: self.logical.checked_add(other.logical)?,
            allocated: self.allocated.checked_add(other.allocated)?,
            reclaimable: self.reclaimable.checked_add(other.reclaimable)?,
            completeness: self.completeness.max(other.completeness),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::ByteSize;

    mod negative {
        use super::ByteSize;
        use crate::{Completeness, Sizes};

        #[test]
        fn partial_plus_complete_is_partial() {
            let partial = Sizes { completeness: Completeness::Partial, ..Sizes::ZERO };

            assert_eq!(partial.checked_add(Sizes::ZERO).unwrap().completeness, Completeness::Partial);
            assert_eq!(Sizes::ZERO.checked_add(partial).unwrap().completeness, Completeness::Partial);
        }

        #[test]
        fn unknown_wins_over_partial() {
            let partial = Sizes { completeness: Completeness::Partial, ..Sizes::ZERO };
            let unknown = Sizes { completeness: Completeness::Unknown, ..Sizes::ZERO };

            assert_eq!(partial.checked_add(unknown).unwrap().completeness, Completeness::Unknown);
        }

        #[test]
        fn checked_add_refuses_an_overflow() {
            assert_eq!(ByteSize::new(u64::MAX).checked_add(ByteSize::new(1)), None);
        }
    }

    mod positive {
        use super::ByteSize;
        use crate::Sizes;

        #[test]
        fn sizes_add_all_three() {
            let a = Sizes::new(ByteSize::new(1), ByteSize::new(10), ByteSize::new(100));
            let b = Sizes::new(ByteSize::new(2), ByteSize::new(20), ByteSize::new(200));

            assert_eq!(a.checked_add(b), Some(Sizes::new(ByteSize::new(3), ByteSize::new(30), ByteSize::new(300))));
        }

        #[test]
        fn a_complete_size_frees_all_it_takes() {
            let sizes = Sizes::complete(ByteSize::new(5), ByteSize::new(4096));

            assert_eq!(sizes.reclaimable, ByteSize::new(4096));
        }

        #[test]
        fn checked_add_sums_two_sizes() {
            assert_eq!(ByteSize::new(2).checked_add(ByteSize::new(3)), Some(ByteSize::new(5)));
        }

        #[test]
        fn displays_the_bytes() {
            assert_eq!(ByteSize::new(42).to_string(), "42 B");
        }

        #[test]
        fn exports_byte_size_as_number() {
            use ts_rs::{Config, TS};

            assert_eq!(ByteSize::inline(&Config::new()), "number");
            assert_eq!(serde_json::to_value(ByteSize::new(42)).unwrap(), 42);
        }

        #[test]
        fn zero_is_the_default() {
            assert_eq!(ByteSize::default(), ByteSize::ZERO);
            assert_eq!(ByteSize::ZERO.bytes(), 0);
        }
    }

    mod property {
        use proptest::prelude::*;

        use super::ByteSize;
        use crate::{Completeness, Sizes};

        fn completeness() -> impl Strategy<Value = Completeness> {
            prop_oneof![Just(Completeness::Complete), Just(Completeness::Partial), Just(Completeness::Unknown)]
        }

        fn sizes() -> impl Strategy<Value = Sizes> {
            (any::<u64>(), any::<u64>(), any::<u64>(), completeness()).prop_map(
                |(logical, allocated, reclaimable, completeness)| Sizes {
                    logical: ByteSize::new(logical),
                    allocated: ByteSize::new(allocated),
                    reclaimable: ByteSize::new(reclaimable),
                    completeness,
                },
            )
        }

        /// Sizes small enough that three of them never overflow.
        fn small_sizes() -> impl Strategy<Value = Sizes> {
            (0..u64::MAX / 4, 0..u64::MAX / 4, 0..u64::MAX / 4, completeness()).prop_map(
                |(logical, allocated, reclaimable, completeness)| Sizes {
                    logical: ByteSize::new(logical),
                    allocated: ByteSize::new(allocated),
                    reclaimable: ByteSize::new(reclaimable),
                    completeness,
                },
            )
        }

        proptest! {
            #[test]
            fn sizes_add_up_or_refuse_an_overflow(a in sizes(), b in sizes()) {
                let logical = a.logical.bytes().checked_add(b.logical.bytes());
                let allocated = a.allocated.bytes().checked_add(b.allocated.bytes());
                let reclaimable = a.reclaimable.bytes().checked_add(b.reclaimable.bytes());
                match a.checked_add(b) {
                    Some(sum) => {
                        prop_assert_eq!(Some(sum.logical.bytes()), logical);
                        prop_assert_eq!(Some(sum.allocated.bytes()), allocated);
                        prop_assert_eq!(Some(sum.reclaimable.bytes()), reclaimable);
                        prop_assert_eq!(sum.completeness, a.completeness.max(b.completeness));
                    }
                    None => prop_assert!(logical.is_none() || allocated.is_none() || reclaimable.is_none()),
                }
            }

            #[test]
            fn checked_add_is_associative_and_keeps_the_weaker_completeness(
                a in small_sizes(), b in small_sizes(), c in small_sizes()
            ) {
                let left = a.checked_add(b).and_then(|ab| ab.checked_add(c));
                let right = b.checked_add(c).and_then(|bc| a.checked_add(bc));

                prop_assert_eq!(left, right);
                let weakest = a.completeness.max(b.completeness).max(c.completeness);
                prop_assert_eq!(left.map(|sum| sum.completeness), Some(weakest));
            }

            #[test]
            fn checked_add_matches_the_integer_sum(a: u64, b: u64) {
                let sum = ByteSize::new(a).checked_add(ByteSize::new(b));
                prop_assert_eq!(sum.map(ByteSize::bytes), a.checked_add(b));
            }
        }
    }
}
