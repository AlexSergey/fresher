//! Reads the team identifier from the code signature embedded in a Mach-O file, from its bytes alone.
//!
//! Supported: thin little-endian Mach-O files (32- and 64-bit) and universal files (32- and 64-bit fat headers).
//! Big-endian thin files are refused: no current macOS executable is one.

use std::io::{Read, Seek, SeekFrom};

/// The most bytes of load commands read.
const MAX_COMMANDS: u64 = 64 * 1024;
/// The most bytes of a signature read.
const MAX_SIGNATURE: u64 = 1024 * 1024;
/// The most slices of a universal file tried.
const MAX_SLICES: u32 = 8;
/// A larger slice count means the file is not a universal file (`0xCAFEBABE` is also the Java class magic).
const MAX_FAT_ARCHS: u32 = 32;
/// The longest team identifier accepted.
const MAX_TEAM_ID: usize = 32;

const FAT_MAGIC: u32 = 0xCAFE_BABE;
const FAT_MAGIC_64: u32 = 0xCAFE_BABF;
const FAT_HEADER_LEN: u64 = 8;
const FAT_ARCH_LEN: u64 = 20;
const FAT_ARCH_64_LEN: u64 = 32;

const MH_MAGIC: u32 = 0xFEED_FACE;
const MH_MAGIC_64: u32 = 0xFEED_FACF;
const MH_HEADER_LEN: u64 = 28;
const MH_HEADER_64_LEN: u64 = 32;

const LOAD_COMMAND_LEN: usize = 8;
const LC_CODE_SIGNATURE: u32 = 0x1D;
const LINKEDIT_DATA_COMMAND_LEN: u32 = 16;

const SUPER_BLOB_MAGIC: u32 = 0xFADE_0CC0;
const SUPER_BLOB_HEADER_LEN: usize = 12;
const BLOB_INDEX_LEN: usize = 8;
const CODE_DIRECTORY_SLOT: u32 = 0;
const CODE_DIRECTORY_MAGIC: u32 = 0xFADE_0C02;
/// The first code directory version with a team offset.
const TEAM_ID_VERSION: u32 = 0x2_0200;
const CODE_DIRECTORY_VERSION_AT: usize = 8;
const CODE_DIRECTORY_TEAM_OFFSET_AT: usize = 48;

/// The team identifier in the code signature of the Mach-O file `file`, if it has one.
///
/// In a universal file the first slice (of at most eight) whose team identifier can be read wins. Only the primary
/// code directory (slot 0) is read; alternate code directories carry the same team identifier. A signature longer
/// than [`MAX_SIGNATURE`] is read in its first bytes only, which hold the team identifier whenever the code directory
/// starts there. The result is 1 to 32 visible ASCII characters; anything else, any I/O error and any malformed
/// data give `None`.
pub(crate) fn team_id(file: &mut (impl Read + Seek)) -> Option<String> {
    let file_len = file.seek(SeekFrom::End(0)).ok()?;
    let magic = be_u32(&read_at(file, 0, 4)?, 0)?;
    match magic {
        FAT_MAGIC => universal_team_id(file, file_len, FAT_ARCH_LEN),
        FAT_MAGIC_64 => universal_team_id(file, file_len, FAT_ARCH_64_LEN),
        _ => slice_team_id(file, 0, file_len),
    }
}

fn universal_team_id<F: Read + Seek>(file: &mut F, file_len: u64, arch_len: u64) -> Option<String> {
    let count = be_u32(&read_at(file, 0, FAT_HEADER_LEN)?, 4)?;
    if count > MAX_FAT_ARCHS {
        return None;
    }
    (0..count.min(MAX_SLICES)).find_map(|index| {
        let at = FAT_HEADER_LEN.checked_add(u64::from(index).checked_mul(arch_len)?)?;
        let arch = read_at(file, at, arch_len)?;
        let (offset, size) = if arch_len == FAT_ARCH_64_LEN {
            (be_u64(&arch, 8)?, be_u64(&arch, 16)?)
        } else {
            (u64::from(be_u32(&arch, 8)?), u64::from(be_u32(&arch, 12)?))
        };
        if offset.checked_add(size)? > file_len {
            return None;
        }
        slice_team_id(file, offset, size)
    })
}

/// The team identifier of the thin Mach-O image at `start`, `len` bytes long.
fn slice_team_id<F: Read + Seek>(file: &mut F, start: u64, len: u64) -> Option<String> {
    let header_len = match le_u32(&read_at(file, start, 4)?, 0)? {
        MH_MAGIC_64 => MH_HEADER_64_LEN,
        MH_MAGIC => MH_HEADER_LEN,
        _ => return None,
    };
    let header = read_at(file, start, header_len)?;
    let command_count = le_u32(&header, 16)?;
    let commands_len = u64::from(le_u32(&header, 20)?);
    if commands_len > MAX_COMMANDS || header_len.checked_add(commands_len)? > len {
        return None;
    }
    let commands = read_at(file, start.checked_add(header_len)?, commands_len)?;
    let (data_offset, data_size) = code_signature_location(&commands, command_count)?;
    if data_size == 0 || data_offset.checked_add(data_size)? > len {
        return None;
    }
    let signature = read_at(file, start.checked_add(data_offset)?, data_size.min(MAX_SIGNATURE))?;
    signature_team_id(&signature)
}

/// The offset and size of the code signature named by the `LC_CODE_SIGNATURE` command among `commands`.
fn code_signature_location(commands: &[u8], count: u32) -> Option<(u64, u64)> {
    let mut at = 0_usize;
    for _ in 0..count {
        let command = le_u32(commands, at)?;
        let command_len = usize::try_from(le_u32(commands, at.checked_add(4)?)?).ok()?;
        if command_len < LOAD_COMMAND_LEN {
            return None;
        }
        let end = at.checked_add(command_len)?;
        let body = commands.get(at..end)?;
        if command == LC_CODE_SIGNATURE {
            if command_len < usize::try_from(LINKEDIT_DATA_COMMAND_LEN).ok()? {
                return None;
            }
            return Some((u64::from(le_u32(body, 8)?), u64::from(le_u32(body, 12)?)));
        }
        at = end;
    }
    None
}

/// The team identifier in the signature super blob `signature` (possibly cut at [`MAX_SIGNATURE`]).
fn signature_team_id(signature: &[u8]) -> Option<String> {
    if be_u32(signature, 0)? != SUPER_BLOB_MAGIC {
        return None;
    }
    let super_len = usize::try_from(be_u32(signature, 4)?).ok()?;
    let super_blob = signature.get(..super_len.min(signature.len()))?;
    let count = be_u32(super_blob, 8)?;
    let code_directory_at = (0..count).find_map(|index| {
        let entry_at = usize::try_from(index).ok()?.checked_mul(BLOB_INDEX_LEN)?.checked_add(SUPER_BLOB_HEADER_LEN)?;
        (be_u32(super_blob, entry_at)? == CODE_DIRECTORY_SLOT)
            .then(|| be_u32(super_blob, entry_at.checked_add(4)?))
            .flatten()
    })?;
    let code_directory = super_blob.get(usize::try_from(code_directory_at).ok()?..)?;
    code_directory_team_id(code_directory)
}

/// The team identifier in the code directory at the start of `blob`.
fn code_directory_team_id(blob: &[u8]) -> Option<String> {
    if be_u32(blob, 0)? != CODE_DIRECTORY_MAGIC {
        return None;
    }
    let directory_len = usize::try_from(be_u32(blob, 4)?).ok()?;
    let directory = blob.get(..directory_len.min(blob.len()))?;
    if be_u32(directory, CODE_DIRECTORY_VERSION_AT)? < TEAM_ID_VERSION {
        return None;
    }
    let team_at = usize::try_from(be_u32(directory, CODE_DIRECTORY_TEAM_OFFSET_AT)?).ok()?;
    if team_at == 0 {
        return None;
    }
    let tail = directory.get(team_at..)?;
    let team_len = tail.iter().take(MAX_TEAM_ID.checked_add(1)?).position(|&byte| byte == 0)?;
    let team = tail.get(..team_len)?;
    if team.is_empty() || !team.iter().all(u8::is_ascii_graphic) {
        return None;
    }
    String::from_utf8(team.to_vec()).ok()
}

/// Exactly `len` bytes of `file` from `offset`; the callers bound `len`.
fn read_at<F: Read + Seek>(file: &mut F, offset: u64, len: u64) -> Option<Vec<u8>> {
    let mut buffer = vec![0; usize::try_from(len).ok()?];
    file.seek(SeekFrom::Start(offset)).ok()?;
    file.read_exact(&mut buffer).ok()?;
    Some(buffer)
}

fn field<const N: usize>(bytes: &[u8], at: usize) -> Option<[u8; N]> {
    bytes.get(at..at.checked_add(N)?)?.try_into().ok()
}

fn be_u32(bytes: &[u8], at: usize) -> Option<u32> {
    field(bytes, at).map(u32::from_be_bytes)
}

fn le_u32(bytes: &[u8], at: usize) -> Option<u32> {
    field(bytes, at).map(u32::from_le_bytes)
}

fn be_u64(bytes: &[u8], at: usize) -> Option<u64> {
    field(bytes, at).map(u64::from_be_bytes)
}

#[cfg(test)]
mod tests;
