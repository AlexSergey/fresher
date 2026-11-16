use std::io::Cursor;

use super::team_id;

const TEAM: &str = "AB12CD34EF";

/// Builds Mach-O byte images with an embedded code signature.
mod image {
    /// The offset of the first load command's `cmdsize` in a [`thin64`] image.
    pub(super) const FIRST_COMMAND_LEN_AT: usize = 36;
    /// The offset of `sizeofcmds` in any thin image.
    pub(super) const COMMANDS_LEN_AT: usize = 20;
    /// The offset of `dataoff` of the code signature command in a [`thin64`] image.
    pub(super) const DATA_OFFSET_AT: usize = 112;
    /// The offset of `teamOffset` in a code directory.
    pub(super) const TEAM_OFFSET_AT: usize = 48;

    const CODE_DIRECTORY_HEADER_LEN: usize = 52;

    pub(super) fn put_le(image: &mut [u8], at: usize, value: u32) {
        image[at..at + 4].copy_from_slice(&value.to_le_bytes());
    }

    pub(super) fn put_be(image: &mut [u8], at: usize, value: u32) {
        image[at..at + 4].copy_from_slice(&value.to_be_bytes());
    }

    fn push_be(bytes: &mut Vec<u8>, value: u32) {
        bytes.extend_from_slice(&value.to_be_bytes());
    }

    fn push_le(bytes: &mut Vec<u8>, value: u32) {
        bytes.extend_from_slice(&value.to_le_bytes());
    }

    fn pad_to(bytes: &mut Vec<u8>, alignment: usize) {
        bytes.resize(bytes.len().next_multiple_of(alignment), 0);
    }

    fn len_u32(len: usize) -> u32 {
        u32::try_from(len).unwrap()
    }

    /// A code directory of `version` whose team field holds the raw bytes `team` (with its NUL, if any).
    pub(super) fn code_directory(version: u32, team: Option<&[u8]>) -> Vec<u8> {
        let ident = b"com.example.tool\0";
        let team_offset = team.map_or(0, |_| CODE_DIRECTORY_HEADER_LEN + ident.len());
        let mut bytes = Vec::new();
        push_be(&mut bytes, 0xFADE_0C02);
        push_be(&mut bytes, 0);
        push_be(&mut bytes, version);
        push_be(&mut bytes, 0);
        push_be(&mut bytes, 0);
        push_be(&mut bytes, len_u32(CODE_DIRECTORY_HEADER_LEN));
        push_be(&mut bytes, 0);
        push_be(&mut bytes, 0);
        push_be(&mut bytes, 0x1000);
        bytes.extend_from_slice(&[32, 2, 0, 12]);
        push_be(&mut bytes, 0);
        push_be(&mut bytes, 0);
        push_be(&mut bytes, len_u32(team_offset));
        bytes.extend_from_slice(ident);
        bytes.extend_from_slice(team.unwrap_or_default());
        let len = len_u32(bytes.len());
        put_be(&mut bytes, 4, len);
        bytes
    }

    /// An empty requirements blob.
    pub(super) fn requirements() -> Vec<u8> {
        let mut bytes = Vec::new();
        push_be(&mut bytes, 0xFADE_0C01);
        push_be(&mut bytes, 12);
        push_be(&mut bytes, 0);
        bytes
    }

    /// A super blob holding `blobs` as `(slot type, blob)` in order.
    pub(super) fn super_blob(blobs: &[(u32, Vec<u8>)]) -> Vec<u8> {
        let index_end = 12 + 8 * blobs.len();
        let total = index_end + blobs.iter().map(|(_, blob)| blob.len()).sum::<usize>();
        let mut bytes = Vec::new();
        push_be(&mut bytes, 0xFADE_0CC0);
        push_be(&mut bytes, len_u32(total));
        push_be(&mut bytes, len_u32(blobs.len()));
        let mut offset = index_end;
        for (slot, blob) in blobs {
            push_be(&mut bytes, *slot);
            push_be(&mut bytes, len_u32(offset));
            offset += blob.len();
        }
        for (_, blob) in blobs {
            bytes.extend_from_slice(blob);
        }
        bytes
    }

    /// The usual signature: requirements first, then the code directory with `team`.
    pub(super) fn signature(team: &str) -> Vec<u8> {
        let team = [team.as_bytes(), b"\0"].concat();
        super_blob(&[(2, requirements()), (0, code_directory(0x2_0400, Some(&team)))])
    }

    fn thin(magic: u32, header_len: usize, segment: u32, segment_len: usize, signature: Option<&[u8]>) -> Vec<u8> {
        let commands_len = segment_len + if signature.is_some() { 16 } else { 0 };
        let data_offset = (header_len + commands_len).next_multiple_of(16);
        let mut bytes = Vec::new();
        push_le(&mut bytes, magic);
        push_le(&mut bytes, 0x0100_000C);
        push_le(&mut bytes, 0);
        push_le(&mut bytes, 2);
        push_le(&mut bytes, if signature.is_some() { 2 } else { 1 });
        push_le(&mut bytes, len_u32(commands_len));
        push_le(&mut bytes, 0);
        pad_to(&mut bytes, header_len);
        push_le(&mut bytes, segment);
        push_le(&mut bytes, len_u32(segment_len));
        bytes.resize(header_len + segment_len, 0);
        if let Some(signature) = signature {
            push_le(&mut bytes, 0x1D);
            push_le(&mut bytes, 16);
            push_le(&mut bytes, len_u32(data_offset));
            push_le(&mut bytes, len_u32(signature.len()));
            pad_to(&mut bytes, 16);
            bytes.extend_from_slice(signature);
        }
        bytes
    }

    /// A thin 64-bit image: a segment command, then the code signature command when `signature` is given.
    pub(super) fn thin64(signature: Option<&[u8]>) -> Vec<u8> {
        thin(0xFEED_FACF, 32, 0x19, 72, signature)
    }

    /// A thin 32-bit image laid out like [`thin64`].
    pub(super) fn thin32(signature: Option<&[u8]>) -> Vec<u8> {
        thin(0xFEED_FACE, 28, 0x1, 56, signature)
    }

    /// A universal image of `slices`, with 64-bit fat entries when `wide`.
    pub(super) fn universal(slices: &[Vec<u8>], wide: bool) -> Vec<u8> {
        let entry_len = if wide { 32 } else { 20 };
        let mut offset = (8 + entry_len * slices.len()).next_multiple_of(16);
        let mut bytes = Vec::new();
        push_be(&mut bytes, if wide { 0xCAFE_BABF } else { 0xCAFE_BABE });
        push_be(&mut bytes, len_u32(slices.len()));
        for slice in slices {
            push_be(&mut bytes, 0x0100_000C);
            push_be(&mut bytes, 0);
            if wide {
                bytes.extend_from_slice(&(offset as u64).to_be_bytes());
                bytes.extend_from_slice(&(slice.len() as u64).to_be_bytes());
                push_be(&mut bytes, 4);
                push_be(&mut bytes, 0);
            } else {
                push_be(&mut bytes, len_u32(offset));
                push_be(&mut bytes, len_u32(slice.len()));
                push_be(&mut bytes, 4);
            }
            offset = (offset + slice.len()).next_multiple_of(16);
        }
        for slice in slices {
            pad_to(&mut bytes, 16);
            bytes.extend_from_slice(slice);
        }
        bytes
    }
}

fn read(image: &[u8]) -> Option<String> {
    team_id(&mut Cursor::new(image))
}

fn thin_with_code_directory(code_directory: Vec<u8>) -> Vec<u8> {
    image::thin64(Some(&image::super_blob(&[(2, image::requirements()), (0, code_directory)])))
}

mod negative {
    use std::io::{self, Read, Seek, SeekFrom};

    use super::{TEAM, image, read, team_id, thin_with_code_directory};

    #[test]
    fn an_empty_file_has_no_team() {
        assert_eq!(read(&[]), None);
    }

    #[test]
    fn a_file_that_is_not_a_mach_o_has_no_team() {
        assert_eq!(read(b"#!/bin/sh\necho hello\n"), None);
    }

    #[test]
    fn a_java_class_is_not_taken_for_a_universal_file() {
        let mut class = vec![0xCA, 0xFE, 0xBA, 0xBE, 0x00, 0x00, 0x00, 0x34];
        class.extend(image::thin64(Some(&image::signature(TEAM))));
        assert_eq!(read(&class), None);
    }

    #[test]
    fn a_universal_file_without_slices_has_no_team() {
        assert_eq!(read(&image::universal(&[], false)), None);
    }

    #[test]
    fn a_universal_slice_beyond_the_file_is_skipped() {
        let mut fat = image::universal(&[image::thin64(Some(&image::signature(TEAM)))], false);
        image::put_be(&mut fat, 20, 0x0010_0000);
        assert_eq!(read(&fat), None);
    }

    #[test]
    fn a_big_endian_thin_file_is_refused() {
        let mut thin = image::thin64(Some(&image::signature(TEAM)));
        thin[..4].copy_from_slice(&0xFEED_FACF_u32.to_be_bytes());
        assert_eq!(read(&thin), None);
    }

    #[test]
    fn a_mach_o_without_a_code_signature_command_has_no_team() {
        assert_eq!(read(&image::thin64(None)), None);
    }

    #[test]
    fn a_signature_beyond_the_file_is_not_read() {
        let mut thin = image::thin64(Some(&image::signature(TEAM)));
        image::put_le(&mut thin, image::DATA_OFFSET_AT, 0x0100_0000);
        assert_eq!(read(&thin), None);
    }

    #[test]
    fn a_truncated_signature_is_not_read() {
        let mut thin = image::thin64(Some(&image::signature(TEAM)));
        thin.truncate(thin.len() - 4);
        assert_eq!(read(&thin), None);
    }

    #[test]
    fn a_super_blob_with_a_wrong_magic_has_no_team() {
        let mut signature = image::signature(TEAM);
        image::put_be(&mut signature, 0, 0xFADE_0CC1);
        assert_eq!(read(&image::thin64(Some(&signature))), None);
    }

    #[test]
    fn a_super_blob_without_a_code_directory_has_no_team() {
        let signature = image::super_blob(&[(2, image::requirements())]);
        assert_eq!(read(&image::thin64(Some(&signature))), None);
    }

    #[test]
    fn a_code_directory_slot_holding_another_blob_has_no_team() {
        let signature = image::super_blob(&[(0, image::requirements())]);
        assert_eq!(read(&image::thin64(Some(&signature))), None);
    }

    #[test]
    fn a_code_directory_older_than_team_ids_has_no_team() {
        let team = [TEAM.as_bytes(), b"\0"].concat();
        let directory = image::code_directory(0x2_0100, Some(&team));
        assert_eq!(read(&thin_with_code_directory(directory)), None);
    }

    #[test]
    fn a_code_directory_with_a_zero_team_offset_has_no_team() {
        assert_eq!(read(&thin_with_code_directory(image::code_directory(0x2_0400, None))), None);
    }

    #[test]
    fn a_team_offset_beyond_the_code_directory_has_no_team() {
        let team = [TEAM.as_bytes(), b"\0"].concat();
        let mut directory = image::code_directory(0x2_0400, Some(&team));
        let len = u32::try_from(directory.len()).unwrap();
        image::put_be(&mut directory, image::TEAM_OFFSET_AT, len + 4);
        assert_eq!(read(&thin_with_code_directory(directory)), None);
    }

    #[test]
    fn a_team_without_a_nul_inside_the_code_directory_has_no_team() {
        let directory = image::code_directory(0x2_0400, Some(TEAM.as_bytes()));
        let signature = image::super_blob(&[(0, directory), (5, vec![0; 16])]);
        assert_eq!(read(&image::thin64(Some(&signature))), None);
    }

    #[test]
    fn an_empty_team_is_no_team() {
        assert_eq!(read(&thin_with_code_directory(image::code_directory(0x2_0400, Some(b"\0")))), None);
    }

    #[test]
    fn a_team_with_non_ascii_bytes_is_no_team() {
        let directory = image::code_directory(0x2_0400, Some("AB12CD34É\0".as_bytes()));
        assert_eq!(read(&thin_with_code_directory(directory)), None);
    }

    #[test]
    fn a_team_longer_than_the_limit_is_no_team() {
        let team = [[b'A'; 33].as_slice(), b"\0"].concat();
        assert_eq!(read(&thin_with_code_directory(image::code_directory(0x2_0400, Some(&team)))), None);
    }

    #[test]
    fn a_load_command_of_zero_size_does_not_loop() {
        let mut thin = image::thin64(Some(&image::signature(TEAM)));
        image::put_le(&mut thin, image::FIRST_COMMAND_LEN_AT, 0);
        assert_eq!(read(&thin), None);
    }

    #[test]
    fn load_commands_over_the_cap_are_not_read() {
        let mut thin = image::thin64(Some(&image::signature(TEAM)));
        thin.resize(256 * 1024, 0);
        image::put_le(&mut thin, image::COMMANDS_LEN_AT, 64 * 1024 + 8);
        assert_eq!(read(&thin), None);
    }

    #[test]
    fn a_load_command_running_past_the_commands_is_refused() {
        let mut thin = image::thin64(Some(&image::signature(TEAM)));
        image::put_le(&mut thin, image::FIRST_COMMAND_LEN_AT, 4096);
        assert_eq!(read(&thin), None);
    }

    #[test]
    fn a_read_error_has_no_team() {
        #[derive(Debug)]
        struct Broken;
        impl Read for Broken {
            fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
                Err(io::Error::other("broken"))
            }
        }
        impl Seek for Broken {
            fn seek(&mut self, _: SeekFrom) -> io::Result<u64> {
                Ok(4096)
            }
        }
        assert_eq!(team_id(&mut Broken), None);
    }
}

mod positive {
    use super::{TEAM, image, read, thin_with_code_directory};

    #[test]
    fn reads_the_team_of_a_thin_64_bit_file() {
        assert_eq!(read(&image::thin64(Some(&image::signature(TEAM)))).as_deref(), Some(TEAM));
    }

    #[test]
    fn reads_the_team_of_a_thin_32_bit_file() {
        assert_eq!(read(&image::thin32(Some(&image::signature(TEAM)))).as_deref(), Some(TEAM));
    }

    #[test]
    fn reads_the_team_of_the_first_signed_slice_of_a_universal_file() {
        let fat = image::universal(&[image::thin64(None), image::thin64(Some(&image::signature(TEAM)))], false);
        assert_eq!(read(&fat).as_deref(), Some(TEAM));
    }

    #[test]
    fn the_first_slice_wins_in_a_universal_file() {
        let slices = [image::thin64(Some(&image::signature(TEAM))), image::thin32(Some(&image::signature("ZZ99")))];
        assert_eq!(read(&image::universal(&slices, false)).as_deref(), Some(TEAM));
    }

    #[test]
    fn reads_the_team_of_a_universal_file_with_64_bit_entries() {
        let fat = image::universal(&[image::thin32(None), image::thin64(Some(&image::signature(TEAM)))], true);
        assert_eq!(read(&fat).as_deref(), Some(TEAM));
    }

    #[test]
    fn finds_the_code_directory_when_it_is_not_the_first_blob() {
        let team = [TEAM.as_bytes(), b"\0"].concat();
        let directory = image::code_directory(0x2_0400, Some(&team));
        let signature = image::super_blob(&[
            (2, image::requirements()),
            (5, vec![0; 24]),
            (0x1000, image::code_directory(0x2_0400, None)),
            (0, directory),
        ]);
        assert_eq!(read(&image::thin64(Some(&signature))).as_deref(), Some(TEAM));
    }

    #[test]
    fn reads_the_team_of_the_first_code_directory_version_with_teams() {
        let team = [TEAM.as_bytes(), b"\0"].concat();
        let directory = image::code_directory(0x2_0200, Some(&team));
        assert_eq!(read(&thin_with_code_directory(directory)).as_deref(), Some(TEAM));
    }

    #[test]
    fn reads_a_team_of_the_longest_accepted_length() {
        let team = "A".repeat(32);
        assert_eq!(read(&image::thin64(Some(&image::signature(&team)))), Some(team));
    }

    #[test]
    #[ignore = "reads real binaries of this Mac; run by hand"]
    fn apple_platform_binaries_have_no_team() {
        for path in ["/bin/ls", "/System/Applications/Calculator.app/Contents/MacOS/Calculator"] {
            let mut file = std::fs::File::open(path).unwrap();
            assert_eq!(super::team_id(&mut file), None, "{path}");
        }
    }
}

mod property {
    use proptest::prelude::*;

    use super::{TEAM, image, read};

    fn valid_images() -> Vec<Vec<u8>> {
        let signed = image::thin64(Some(&image::signature(TEAM)));
        vec![
            signed.clone(),
            image::thin32(Some(&image::signature(TEAM))),
            image::universal(&[image::thin64(None), signed.clone()], false),
            image::universal(&[signed], true),
        ]
    }

    proptest! {
        #[test]
        fn random_bytes_never_panic(bytes in proptest::collection::vec(any::<u8>(), 0..4096)) {
            let _ = read(&bytes);
        }

        #[test]
        fn random_byte_mutations_of_a_valid_image_never_panic(
            which in 0..4_usize,
            mutations in proptest::collection::vec((any::<usize>(), any::<u8>()), 1..16),
        ) {
            let mut bytes = valid_images().swap_remove(which);
            for (at, value) in mutations {
                let len = bytes.len();
                bytes[at % len] = value;
            }
            let _ = read(&bytes);
        }

        #[test]
        fn random_word_mutations_of_a_valid_image_never_panic(
            which in 0..4_usize,
            mutations in proptest::collection::vec((any::<usize>(), any::<u32>()), 1..8),
        ) {
            let mut bytes = valid_images().swap_remove(which);
            for (at, value) in mutations {
                let at = (at % (bytes.len() / 4)) * 4;
                image::put_be(&mut bytes, at, value);
            }
            let _ = read(&bytes);
        }

        #[test]
        fn truncations_of_a_valid_image_never_panic(which in 0..4_usize, keep in any::<usize>()) {
            let mut bytes = valid_images().swap_remove(which);
            let len = bytes.len();
            bytes.truncate(keep % (len + 1));
            let _ = read(&bytes);
        }
    }
}
