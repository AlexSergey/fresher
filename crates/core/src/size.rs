use std::fmt;

/// A number of bytes on disk.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ByteSize(u64);

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

#[cfg(test)]
mod tests {
    use super::ByteSize;

    mod negative {
        use super::ByteSize;

        #[test]
        fn checked_add_refuses_an_overflow() {
            assert_eq!(ByteSize::new(u64::MAX).checked_add(ByteSize::new(1)), None);
        }
    }

    mod positive {
        use super::ByteSize;

        #[test]
        fn checked_add_sums_two_sizes() {
            assert_eq!(ByteSize::new(2).checked_add(ByteSize::new(3)), Some(ByteSize::new(5)));
        }

        #[test]
        fn displays_the_bytes() {
            assert_eq!(ByteSize::new(42).to_string(), "42 B");
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

        proptest! {
            #[test]
            fn checked_add_matches_the_integer_sum(a: u64, b: u64) {
                let sum = ByteSize::new(a).checked_add(ByteSize::new(b));
                prop_assert_eq!(sum.map(ByteSize::bytes), a.checked_add(b));
            }
        }
    }
}
