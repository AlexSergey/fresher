//! A hash that stays the same across Rust versions and machines, for keys written to disk. `std`'s hasher is not
//! stable, so it is not used for them.

const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

/// The 64-bit FNV-1a hash of `bytes`. Not for security: only for keys and fingerprints.
pub fn stable_hash(bytes: &[u8]) -> u64 {
    bytes.iter().fold(FNV_OFFSET_BASIS, |hash, &byte| (hash ^ u64::from(byte)).wrapping_mul(FNV_PRIME))
}

#[cfg(test)]
mod tests {
    mod negative {
        use crate::stable_hash;

        #[test]
        fn a_one_byte_change_changes_the_hash() {
            assert_ne!(stable_hash(b"/Users/a"), stable_hash(b"/Users/b"));
            assert_ne!(stable_hash(b"xy"), stable_hash(b"yx"));
        }
    }

    mod positive {
        use crate::stable_hash;

        #[test]
        fn stable_hash_matches_the_fnv_1a_test_vectors() {
            assert_eq!(stable_hash(b""), 0xcbf2_9ce4_8422_2325);
            assert_eq!(stable_hash(b"a"), 0xaf63_dc4c_8601_ec8c);
            assert_eq!(stable_hash(b"foobar"), 0x8594_4171_f739_67e8);
        }
    }
}
