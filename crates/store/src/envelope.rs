//! The binary envelope of a scan cache file: a fixed header, then the body compressed with deflate level 1.
//!
//! The header, little-endian: `magic` (8 bytes, `FRSHSCAN`), `format_version` (u16), `scope` (u64), `stored_len`
//! (u64, the length of the compressed bytes that follow) and `crc32` (u32, of those compressed bytes). The body
//! itself is the scanner's; the envelope only keeps it whole and tells whose it is.

use fresher_core::ScopeKey;
use miniz_oxide::deflate::compress_to_vec;
use miniz_oxide::inflate::{TINFLStatus, decompress_to_vec_with_limit};

/// The version of the envelope this build writes and reads.
pub(crate) const FORMAT_VERSION: u16 = 1;
const MAGIC: [u8; 8] = *b"FRSHSCAN";
/// The length of the header: magic, version, scope, stored length and checksum.
pub(crate) const HEADER_LEN: usize = 8 + 2 + 8 + 8 + 4;
/// The fastest deflate level: measured in plan 004/01 at −60 % on a home folder's body for 49 ms.
const DEFLATE_LEVEL: u8 = 1;

/// A file read through the envelope.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Opened {
    /// The body, checked and inflated.
    Body(Vec<u8>),
    /// The file is not a usable cache of the scope: it costs one full scan, never a wrong result.
    Discarded(Reason),
}

/// Why a cache file was not used. Logged; it holds no path.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Reason {
    /// The file is shorter than its header.
    Short,
    /// The file does not start with the magic.
    NotACacheFile,
    /// The file is from a newer version of the app.
    NewerFormat,
    /// The file is from an older version of the app: the cache is rebuilt, not migrated.
    OlderFormat,
    /// The bytes after the header are not as many as the header says.
    LengthMismatch,
    /// The checksum of the bytes after the header does not match.
    Checksum,
    /// The file holds the cache of another scope.
    OtherScope,
    /// The bytes after the header are not a deflate stream.
    NotDeflate,
    /// The body inflates past the limit.
    TooLarge,
}

struct Header {
    scope: u64,
    stored_len: u64,
    crc32: u32,
}

/// The file for `body` of `scope`: the header, then the body compressed.
pub(crate) fn seal(scope: ScopeKey, body: &[u8]) -> Vec<u8> {
    let stored = compress_to_vec(body, DEFLATE_LEVEL);
    let mut bytes = Vec::with_capacity(HEADER_LEN.saturating_add(stored.len()));
    bytes.extend_from_slice(&MAGIC);
    bytes.extend_from_slice(&FORMAT_VERSION.to_le_bytes());
    bytes.extend_from_slice(&scope.get().to_le_bytes());
    bytes.extend_from_slice(&length(&stored).to_le_bytes());
    bytes.extend_from_slice(&crc32fast::hash(&stored).to_le_bytes());
    bytes.extend_from_slice(&stored);
    bytes
}

/// The body in `bytes` if they are a whole, current file of `scope` whose body inflates to at most `limit` bytes.
pub(crate) fn open(bytes: &[u8], scope: ScopeKey, limit: usize) -> Opened {
    let (header, stored) = match read_header(bytes) {
        Ok(read) => read,
        Err(reason) => return Opened::Discarded(reason),
    };
    if length(stored) != header.stored_len {
        return Opened::Discarded(Reason::LengthMismatch);
    }
    if crc32fast::hash(stored) != header.crc32 {
        return Opened::Discarded(Reason::Checksum);
    }
    if header.scope != scope.get() {
        return Opened::Discarded(Reason::OtherScope);
    }
    match decompress_to_vec_with_limit(stored, limit) {
        Ok(body) => Opened::Body(body),
        Err(error) if error.status == TINFLStatus::HasMoreOutput => Opened::Discarded(Reason::TooLarge),
        Err(_) => Opened::Discarded(Reason::NotDeflate),
    }
}

/// The scope a file of the current format is of, read from its header alone; `None` for anything else.
pub(crate) fn scope_of(header: &[u8]) -> Option<ScopeKey> {
    read_header(header).ok().map(|(header, _)| ScopeKey::new(header.scope))
}

fn read_header(bytes: &[u8]) -> Result<(Header, &[u8]), Reason> {
    let (magic, rest) = bytes.split_first_chunk::<8>().ok_or(Reason::Short)?;
    if *magic != MAGIC {
        return Err(Reason::NotACacheFile);
    }
    let (version, rest) = rest.split_first_chunk::<2>().ok_or(Reason::Short)?;
    match u16::from_le_bytes(*version).cmp(&FORMAT_VERSION) {
        std::cmp::Ordering::Greater => return Err(Reason::NewerFormat),
        std::cmp::Ordering::Less => return Err(Reason::OlderFormat),
        std::cmp::Ordering::Equal => {}
    }
    let (scope, rest) = rest.split_first_chunk::<8>().ok_or(Reason::Short)?;
    let (stored_len, rest) = rest.split_first_chunk::<8>().ok_or(Reason::Short)?;
    let (crc32, stored) = rest.split_first_chunk::<4>().ok_or(Reason::Short)?;
    let header = Header {
        scope: u64::from_le_bytes(*scope),
        stored_len: u64::from_le_bytes(*stored_len),
        crc32: u32::from_le_bytes(*crc32),
    };
    Ok((header, stored))
}

fn length(bytes: &[u8]) -> u64 {
    u64::try_from(bytes.len()).unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use fresher_core::ScopeKey;

    const SCOPE: ScopeKey = ScopeKey::new(0x5c09e);
    const LIMIT: usize = 1 << 20;
    const BODY: &[u8] = b"the scanner's body, the scanner's body, the scanner's body";

    mod negative {
        use fresher_core::ScopeKey;

        use super::{BODY, LIMIT, SCOPE};
        use crate::envelope::{FORMAT_VERSION, HEADER_LEN, Opened, Reason, open, scope_of, seal};

        fn with_version(version: u16) -> Vec<u8> {
            let mut bytes = seal(SCOPE, BODY);
            bytes[8..10].copy_from_slice(&version.to_le_bytes());
            bytes
        }

        #[test]
        fn a_truncated_file_is_discarded() {
            let bytes = seal(SCOPE, BODY);

            for length in 0..bytes.len() {
                let opened = open(&bytes[..length], SCOPE, LIMIT);

                assert!(matches!(opened, Opened::Discarded(_)), "{length}: {opened:?}");
            }
        }

        #[test]
        fn a_flipped_byte_is_discarded() {
            let bytes = seal(SCOPE, BODY);

            for index in 0..bytes.len() {
                let mut flipped = bytes.clone();
                flipped[index] ^= 0x01;

                let opened = open(&flipped, SCOPE, LIMIT);

                assert!(matches!(opened, Opened::Discarded(_)), "{index}: {opened:?}");
            }
        }

        #[test]
        fn a_trailing_byte_is_a_length_mismatch() {
            let mut bytes = seal(SCOPE, BODY);
            bytes.push(0);

            assert_eq!(open(&bytes, SCOPE, LIMIT), Opened::Discarded(Reason::LengthMismatch));
        }

        #[test]
        fn another_magic_is_not_a_cache_file() {
            let mut bytes = seal(SCOPE, BODY);
            bytes[0] = b'X';

            assert_eq!(open(&bytes, SCOPE, LIMIT), Opened::Discarded(Reason::NotACacheFile));
            assert_eq!(scope_of(&bytes), None);
        }

        #[test]
        fn another_scopes_file_is_not_opened() {
            let bytes = seal(ScopeKey::new(7), BODY);

            assert_eq!(open(&bytes, SCOPE, LIMIT), Opened::Discarded(Reason::OtherScope));
        }

        #[test]
        fn a_newer_or_older_format_is_discarded() {
            let newer = with_version(FORMAT_VERSION + 1);
            let older = with_version(FORMAT_VERSION - 1);

            assert_eq!(open(&newer, SCOPE, LIMIT), Opened::Discarded(Reason::NewerFormat));
            assert_eq!(open(&older, SCOPE, LIMIT), Opened::Discarded(Reason::OlderFormat));
            assert_eq!(scope_of(&newer[..HEADER_LEN]), None);
        }

        #[test]
        fn a_body_that_inflates_past_the_limit_is_discarded() {
            let bytes = seal(SCOPE, &[0_u8; 4096]);

            assert_eq!(open(&bytes, SCOPE, 4095), Opened::Discarded(Reason::TooLarge));
        }

        #[test]
        fn bytes_that_are_not_deflate_are_discarded() {
            let stored = [0xff_u8; 16];
            let mut bytes = seal(SCOPE, b"")[..HEADER_LEN].to_vec();
            bytes[18..26].copy_from_slice(&16_u64.to_le_bytes());
            bytes[26..30].copy_from_slice(&crc32fast::hash(&stored).to_le_bytes());
            bytes.extend_from_slice(&stored);

            assert_eq!(open(&bytes, SCOPE, LIMIT), Opened::Discarded(Reason::NotDeflate));
        }
    }

    mod positive {
        use super::{BODY, LIMIT, SCOPE};
        use crate::envelope::{HEADER_LEN, Opened, open, scope_of, seal};

        #[test]
        fn a_sealed_body_opens_unchanged() {
            for body in [&b""[..], BODY, &vec![7_u8; 100_000]] {
                assert_eq!(open(&seal(SCOPE, body), SCOPE, LIMIT), Opened::Body(body.to_vec()));
            }
        }

        #[test]
        fn a_body_of_exactly_the_limit_opens() {
            let body = vec![3_u8; 4096];

            assert_eq!(open(&seal(SCOPE, &body), SCOPE, 4096), Opened::Body(body));
        }

        #[test]
        fn a_compressible_body_is_stored_smaller() {
            let body = BODY.repeat(1000);

            assert!(seal(SCOPE, &body).len() < body.len() / 10);
        }

        #[test]
        fn the_header_starts_with_the_magic_and_names_the_scope() {
            let bytes = seal(SCOPE, BODY);

            assert_eq!(&bytes[..10], b"FRSHSCAN\x01\x00");
            assert_eq!(scope_of(&bytes[..HEADER_LEN]), Some(SCOPE));
        }
    }
}
