//! The process table (`libproc`): the processes, their owners and names, the files they hold open and the files
//! they map. Only reads; nothing here can change another process.

use std::ffi::OsString;
use std::io;
use std::os::unix::ffi::OsStringExt as _;
use std::path::PathBuf;
use std::ptr;

use libc::{c_char, c_int, pid_t};

/// `PROC_PIDFDVNODEPATHINFO` of the SDK's `<sys/proc_info.h>`: a vnode descriptor with its path.
const PROC_PIDFDVNODEPATHINFO: c_int = 2;
/// `PROC_PIDREGIONPATHINFO` of `<sys/proc_info.h>`: the mapped region at or after an address, with its file's path.
const PROC_PIDREGIONPATHINFO: c_int = 8;
/// Room for what appears between asking a list's size and reading it.
const SPARE: usize = 64;
/// How often a list that filled its whole buffer is read again with a larger one.
const ATTEMPTS: usize = 4;
/// The longest name `proc_name` gives (`2 * MAXCOMLEN`), with room to spare.
const NAME: usize = 64;

/// `struct proc_fileinfo` of `<sys/proc_info.h>` (`libc` 0.2.191 lacks it).
#[repr(C)]
#[expect(clippy::struct_field_names, reason = "the field names of the SDK's structure")]
struct ProcFileInfo {
    fi_openflags: u32,
    fi_status: u32,
    fi_offset: libc::off_t,
    fi_type: i32,
    fi_guardflags: u32,
}

/// `struct vnode_fdinfowithpath` of `<sys/proc_info.h>`.
#[repr(C)]
struct VnodeFdInfoWithPath {
    pfi: ProcFileInfo,
    pvip: libc::vnode_info_path,
}

/// `struct proc_regioninfo` of `<sys/proc_info.h>`.
#[repr(C)]
#[expect(clippy::struct_field_names, reason = "the field names of the SDK's structure")]
struct ProcRegionInfo {
    pri_protection: u32,
    pri_max_protection: u32,
    pri_inheritance: u32,
    pri_flags: u32,
    pri_offset: u64,
    pri_behavior: u32,
    pri_user_wired_count: u32,
    pri_user_tag: u32,
    pri_pages_resident: u32,
    pri_pages_shared_now_private: u32,
    pri_pages_swapped_out: u32,
    pri_pages_dirtied: u32,
    pri_ref_count: u32,
    pri_shadow_depth: u32,
    pri_share_mode: u32,
    pri_private_pages_resident: u32,
    pri_shared_pages_resident: u32,
    pri_obj_id: u32,
    pri_depth: u32,
    pri_address: u64,
    pri_size: u64,
}

/// `struct proc_regionwithpathinfo` of `<sys/proc_info.h>`.
#[repr(C)]
struct ProcRegionWithPathInfo {
    prp_prinfo: ProcRegionInfo,
    prp_vip: libc::vnode_info_path,
}

// The sizes `<sys/proc_info.h>` gives these structures on both architectures.
const _: () = assert!(size_of::<ProcFileInfo>() == 24);
const _: () = assert!(size_of::<ProcRegionInfo>() == 96);

/// The real user id of this process.
pub(crate) fn user_id() -> u32 {
    // SAFETY: `getuid` takes nothing, touches no memory of ours and always succeeds.
    unsafe { libc::getuid() }
}

/// Every process id of the system.
pub(crate) fn all_pids() -> io::Result<Vec<pid_t>> {
    // SAFETY: a null buffer of size 0 asks only for the number of processes.
    let count = unsafe { libc::proc_listallpids(ptr::null_mut(), 0) };
    let mut capacity = usize::try_from(count).map_err(|_| io::Error::last_os_error())? + SPARE;
    for _ in 0..ATTEMPTS {
        let mut pids: Vec<pid_t> = vec![0; capacity];
        let bytes = buffer_size(size_of_val(pids.as_slice()))?;
        // SAFETY: `pids` is writable for exactly `bytes` bytes, a whole number of `pid_t`.
        let filled = unsafe { libc::proc_listallpids(pids.as_mut_ptr().cast(), bytes) };
        let filled = usize::try_from(filled).map_err(|_| io::Error::last_os_error())?;
        if filled < capacity {
            pids.truncate(filled);
            return Ok(pids);
        }
        capacity *= 2;
    }
    Err(io::Error::other("the process list kept growing"))
}

/// The owner of `pid`. Another user's process answers `EPERM`, a process that ended `ESRCH`.
pub(crate) fn owner(pid: pid_t) -> io::Result<u32> {
    // SAFETY: `proc_bsdinfo` holds integers and arrays of integers only, for which all-zero bytes are valid.
    let mut info: libc::proc_bsdinfo = unsafe { std::mem::zeroed() };
    let size = buffer_size(size_of::<libc::proc_bsdinfo>())?;
    // SAFETY: `info` is writable for exactly `size` bytes and alive until the call returns.
    let written = unsafe { libc::proc_pidinfo(pid, libc::PROC_PIDTBSDINFO, 0, (&raw mut info).cast(), size) };
    whole(written, size)?;
    Ok(info.pbi_uid)
}

/// The short name of `pid` (an app's helper has its own), if the system gives one.
pub(crate) fn short_name(pid: pid_t) -> Option<String> {
    let mut buffer = [0_u8; NAME];
    let size = u32::try_from(buffer.len()).ok()?;
    // SAFETY: the call writes at most `size` bytes into `buffer`, which is that long and alive until it returns.
    let length = unsafe { libc::proc_name(pid, buffer.as_mut_ptr().cast(), size) };
    let name = buffer.get(..usize::try_from(length).ok()?)?;
    (!name.is_empty()).then(|| String::from_utf8_lossy(name).into_owned())
}

/// The executable's path of `pid`, if the system gives one.
pub(crate) fn executable(pid: pid_t) -> Option<PathBuf> {
    let mut buffer = vec![0_u8; usize::try_from(libc::PROC_PIDPATHINFO_MAXSIZE).ok()?];
    let size = u32::try_from(buffer.len()).ok()?;
    // SAFETY: the call writes at most `size` bytes into `buffer`, which is that long and alive until it returns.
    let length = unsafe { libc::proc_pidpath(pid, buffer.as_mut_ptr().cast(), size) };
    buffer.truncate(usize::try_from(length).ok()?);
    (!buffer.is_empty()).then(|| PathBuf::from(OsString::from_vec(buffer)))
}

/// The descriptors of `pid` that are vnodes (files and folders).
pub(crate) fn vnode_descriptors(pid: pid_t) -> io::Result<Vec<c_int>> {
    const ENTRY: usize = size_of::<libc::proc_fdinfo>();
    // SAFETY: a null buffer of size 0 asks only for the size the list needs.
    let needed = unsafe { libc::proc_pidinfo(pid, libc::PROC_PIDLISTFDS, 0, ptr::null_mut(), 0) };
    let mut capacity = usize::try_from(needed).map_err(|_| io::Error::last_os_error())? / ENTRY + SPARE;
    for _ in 0..ATTEMPTS {
        let mut list = vec![libc::proc_fdinfo { proc_fd: 0, proc_fdtype: 0 }; capacity];
        let bytes = buffer_size(size_of_val(list.as_slice()))?;
        // SAFETY: `list` is writable for exactly `bytes` bytes, a whole number of `proc_fdinfo`.
        let written = unsafe { libc::proc_pidinfo(pid, libc::PROC_PIDLISTFDS, 0, list.as_mut_ptr().cast(), bytes) };
        let filled =
            usize::try_from(written).ok().filter(|written| *written > 0).ok_or_else(io::Error::last_os_error)?;
        if filled / ENTRY < capacity {
            list.truncate(filled / ENTRY);
            let vnode = u32::try_from(libc::PROX_FDTYPE_VNODE).unwrap_or(u32::MAX);
            return Ok(list.iter().filter(|entry| entry.proc_fdtype == vnode).map(|entry| entry.proc_fd).collect());
        }
        capacity *= 2;
    }
    Err(io::Error::other("the descriptor list kept growing"))
}

/// The path of descriptor `fd` of `pid`; `None` for a vnode with no path. A path the system withholds answers
/// `EPERM`; a descriptor closed meanwhile `EBADF`.
pub(crate) fn descriptor_path(pid: pid_t, fd: c_int) -> io::Result<Option<PathBuf>> {
    // SAFETY: the structure holds integers and arrays of integers only, for which all-zero bytes are valid.
    let mut info: VnodeFdInfoWithPath = unsafe { std::mem::zeroed() };
    let size = buffer_size(size_of::<VnodeFdInfoWithPath>())?;
    // SAFETY: `info` is writable for exactly `size` bytes and alive until the call returns.
    let written = unsafe { libc::proc_pidfdinfo(pid, fd, PROC_PIDFDVNODEPATHINFO, (&raw mut info).cast(), size) };
    whole(written, size)?;
    Ok(path_from(&info.pvip.vip_path))
}

/// The working folder of `pid`.
pub(crate) fn working_folder(pid: pid_t) -> io::Result<Option<PathBuf>> {
    // SAFETY: the structure holds integers and arrays of integers only, for which all-zero bytes are valid.
    let mut info: libc::proc_vnodepathinfo = unsafe { std::mem::zeroed() };
    let size = buffer_size(size_of::<libc::proc_vnodepathinfo>())?;
    // SAFETY: `info` is writable for exactly `size` bytes and alive until the call returns.
    let written = unsafe { libc::proc_pidinfo(pid, libc::PROC_PIDVNODEPATHINFO, 0, (&raw mut info).cast(), size) };
    whole(written, size)?;
    Ok(path_from(&info.pvi_cdir.vip_path))
}

/// Every file `pid` maps, each once: one walk over its regions (hundreds of them; a file mapped without a
/// descriptor, such as a database, is seen only here).
pub(crate) fn mapped_files(pid: pid_t) -> io::Result<Vec<PathBuf>> {
    let size = buffer_size(size_of::<ProcRegionWithPathInfo>())?;
    let mut files = Vec::new();
    let mut address = 0_u64;
    loop {
        // SAFETY: the structure holds integers and arrays of integers only, for which all-zero bytes are valid.
        let mut region: ProcRegionWithPathInfo = unsafe { std::mem::zeroed() };
        // SAFETY: `region` is writable for exactly `size` bytes and alive until the call returns.
        let written =
            unsafe { libc::proc_pidinfo(pid, PROC_PIDREGIONPATHINFO, address, (&raw mut region).cast(), size) };
        if written != size {
            // The first region refused is the answer (`EPERM`, `ESRCH`); a later one is the end of the walk.
            if address == 0 {
                whole(written, size)?;
            }
            break;
        }
        files.extend(path_from(&region.prp_vip.vip_path));
        let next = region.prp_prinfo.pri_address.saturating_add(region.prp_prinfo.pri_size);
        if next <= address {
            break;
        }
        address = next;
    }
    files.sort_unstable();
    files.dedup();
    Ok(files)
}

/// A buffer's length as the calls take it.
fn buffer_size(bytes: usize) -> io::Result<c_int> {
    c_int::try_from(bytes).map_err(|_| io::Error::from(io::ErrorKind::InvalidInput))
}

/// A call that fills a structure succeeded only when it wrote the whole of it; 0 sets `errno`.
fn whole(written: c_int, size: c_int) -> io::Result<()> {
    match written {
        0 => Err(io::Error::last_os_error()),
        written if written == size => Ok(()),
        _ => Err(io::Error::from(io::ErrorKind::InvalidData)),
    }
}

/// The NUL-terminated path of a `vnode_info_path`; `None` when it is empty.
fn path_from(raw: &[[c_char; 32]; 32]) -> Option<PathBuf> {
    let bytes: Vec<u8> =
        raw.as_flattened().iter().map(|byte| byte.cast_unsigned()).take_while(|byte| *byte != 0).collect();
    (!bytes.is_empty()).then(|| PathBuf::from(OsString::from_vec(bytes)))
}

/// A file mapped into this process, unmapped when dropped (tests only).
#[cfg(test)]
#[derive(Debug)]
pub(crate) struct Mapping {
    address: ptr::NonNull<libc::c_void>,
    length: usize,
}

#[cfg(test)]
impl Mapping {
    /// Maps the first `length` bytes of `file`, read-only; the descriptor may be closed afterwards.
    pub(crate) fn of(file: &std::fs::File, length: usize) -> io::Result<Self> {
        use std::os::fd::AsRawFd as _;
        // SAFETY: a fresh private, read-only mapping at an address the system picks; `file` is open for reading.
        let address =
            unsafe { libc::mmap(ptr::null_mut(), length, libc::PROT_READ, libc::MAP_PRIVATE, file.as_raw_fd(), 0) };
        if address == libc::MAP_FAILED {
            return Err(io::Error::last_os_error());
        }
        let address = ptr::NonNull::new(address).ok_or_else(|| io::Error::from(io::ErrorKind::InvalidData))?;
        Ok(Self { address, length })
    }
}

#[cfg(test)]
impl Drop for Mapping {
    fn drop(&mut self) {
        // SAFETY: the region was mapped by `Mapping::of` with this length, and nothing borrows it.
        unsafe { libc::munmap(self.address.as_ptr(), self.length) };
    }
}
