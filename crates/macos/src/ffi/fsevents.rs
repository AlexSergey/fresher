//! The change journal: `FSEvents` (`CoreServices`), with the few `CoreFoundation` and libdispatch calls around it.
//!
//! Every object created here is owned by a value whose `Drop` releases it, so no path leaks one.

use std::ffi::{CStr, CString, c_char, c_void};
use std::io;
use std::ptr::{self, NonNull};
use std::sync::{Condvar, Mutex, PoisonError};

type CfIndex = isize;
type CfTypeRef = *const c_void;
type FsEventStreamRef = *mut c_void;
type DispatchQueue = *mut c_void;
type Callback = extern "C" fn(
    stream: *const c_void,
    info: *mut c_void,
    count: usize,
    paths: *mut c_void,
    flags: *const u32,
    ids: *const u64,
);

#[repr(C)]
struct StreamContext {
    version: CfIndex,
    info: *mut c_void,
    retain: *const c_void,
    release: *const c_void,
    copy_description: *const c_void,
}

/// `CFArrayCallBacks`; only its address is used.
#[repr(C)]
struct ArrayCallBacks {
    version: CfIndex,
    retain: *const c_void,
    release: *const c_void,
    copy_description: *const c_void,
    equal: *const c_void,
}

#[repr(C)]
struct UuidBytes {
    bytes: [u8; 16],
}

// SAFETY: the declarations match `<CoreFoundation/CFArray.h>`, `CFString.h`, `CFUUID.h` and `CFBase.h`.
#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    static kCFTypeArrayCallBacks: ArrayCallBacks;
    fn CFStringCreateWithFileSystemRepresentation(allocator: CfTypeRef, path: *const c_char) -> CfTypeRef;
    fn CFArrayCreate(
        allocator: CfTypeRef,
        values: *const CfTypeRef,
        count: CfIndex,
        callbacks: *const ArrayCallBacks,
    ) -> CfTypeRef;
    fn CFUUIDGetUUIDBytes(uuid: CfTypeRef) -> UuidBytes;
    fn CFRelease(object: CfTypeRef);
}

// SAFETY: the declarations match `<CoreServices/.../FSEvents.h>`.
#[link(name = "CoreServices", kind = "framework")]
unsafe extern "C" {
    fn FSEventsGetCurrentEventId() -> u64;
    fn FSEventsCopyUUIDForDevice(device: libc::dev_t) -> CfTypeRef;
    fn FSEventStreamCreate(
        allocator: CfTypeRef,
        callback: Callback,
        context: *const StreamContext,
        paths: CfTypeRef,
        since: u64,
        latency: f64,
        flags: u32,
    ) -> FsEventStreamRef;
    fn FSEventStreamSetDispatchQueue(stream: FsEventStreamRef, queue: DispatchQueue);
    fn FSEventStreamStart(stream: FsEventStreamRef) -> u8;
    fn FSEventStreamFlushSync(stream: FsEventStreamRef);
    fn FSEventStreamStop(stream: FsEventStreamRef);
    fn FSEventStreamInvalidate(stream: FsEventStreamRef);
    fn FSEventStreamRelease(stream: FsEventStreamRef);
}

// SAFETY: the declarations match `<dispatch/queue.h>` and `<dispatch/object.h>`; libdispatch is part of libSystem.
unsafe extern "C" {
    fn dispatch_queue_create(label: *const c_char, attributes: *const c_void) -> DispatchQueue;
    fn dispatch_sync_f(queue: DispatchQueue, context: *mut c_void, work: extern "C" fn(*mut c_void));
    fn dispatch_release(object: DispatchQueue);
}

/// `kFSEventStreamCreateFlagNoDefer | kFSEventStreamCreateFlagWatchRoot`: folder-level events, delivered at once, and
/// a change of the path to a watched root reported.
const CREATE_FLAGS: u32 = 0x2 | 0x4;

/// The id of the newest event the system has recorded, on any volume.
pub(crate) fn current_event_id() -> u64 {
    // SAFETY: no arguments; the call only reads the system's counter.
    unsafe { FSEventsGetCurrentEventId() }
}

/// The UUID of the journal the system keeps for `device`; `None` for a volume with no journal.
pub(crate) fn journal_id(device: libc::dev_t) -> Option<u128> {
    // SAFETY: a plain integer in; the result is a new reference we own, or null.
    let uuid = CfOwned::new(unsafe { FSEventsCopyUUIDForDevice(device) })?;
    // SAFETY: `uuid` is a live `CFUUIDRef`; the bytes are returned by value.
    let bytes = unsafe { CFUUIDGetUUIDBytes(uuid.0.as_ptr()) };
    Some(u128::from_be_bytes(bytes.bytes))
}

/// One event of a stream: a folder's path, what happened, and its id.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Event {
    pub(crate) path: Vec<u8>,
    pub(crate) flags: u32,
    pub(crate) id: u64,
}

/// What the stream's callback hands over: the events so far, and whether the history is complete.
#[derive(Debug, Default)]
pub(crate) struct Received {
    pub(crate) events: Vec<Event>,
    pub(crate) history_done: bool,
}

/// The state shared with the callback, which runs on the stream's queue.
#[derive(Debug, Default)]
pub(crate) struct Inbox {
    pub(crate) received: Mutex<Received>,
    pub(crate) arrived: Condvar,
}

/// `kFSEventStreamEventFlagHistoryDone`.
pub(crate) const HISTORY_DONE: u32 = 0x10;

extern "C" fn on_events(
    _stream: *const c_void,
    info: *mut c_void,
    count: usize,
    paths: *mut c_void,
    flags: *const u32,
    ids: *const u64,
) {
    // SAFETY: `info` is the `Inbox` the stream was created with; the stream that calls us is invalidated, and its
    // queue drained, before the `Inbox` is dropped.
    let inbox = unsafe { &*info.cast::<Inbox>() };
    if count == 0 || paths.is_null() || flags.is_null() || ids.is_null() {
        return;
    }
    // SAFETY: the stream passes `count` paths (C strings, without `kFSEventStreamCreateFlagUseCFTypes`), alive for
    // the duration of the callback.
    let paths = unsafe { std::slice::from_raw_parts(paths.cast::<*const c_char>(), count) };
    // SAFETY: the stream passes `count` flags, alive for the duration of the callback.
    let flags = unsafe { std::slice::from_raw_parts(flags, count) };
    // SAFETY: the stream passes `count` ids, alive for the duration of the callback.
    let ids = unsafe { std::slice::from_raw_parts(ids, count) };
    let mut received = inbox.received.lock().unwrap_or_else(PoisonError::into_inner);
    for ((&path, &flags), &id) in paths.iter().zip(flags).zip(ids) {
        if path.is_null() {
            continue;
        }
        // SAFETY: each path is a NUL-terminated string that lives for the duration of the callback.
        let path = unsafe { CStr::from_ptr(path) }.to_bytes().to_vec();
        received.history_done |= flags & HISTORY_DONE != 0;
        received.events.push(Event { path, flags, id });
    }
    drop(received);
    inbox.arrived.notify_all();
}

extern "C" fn nothing(_context: *mut c_void) {}

/// A started `FSEvents` stream on its own serial queue, replaying the history of some paths since an event id.
#[derive(Debug)]
pub(crate) struct Stream {
    raw: NonNull<c_void>,
    queue: NonNull<c_void>,
    inbox: Box<Inbox>,
}

impl Stream {
    /// Creates and starts a stream for `paths` that replays every change since `since`.
    pub(crate) fn start(paths: &[CString], since: u64) -> io::Result<Self> {
        let paths = path_array(paths)?;
        let inbox = Box::new(Inbox::default());
        let context = StreamContext {
            version: 0,
            info: ptr::from_ref(&*inbox).cast_mut().cast(),
            retain: ptr::null(),
            release: ptr::null(),
            copy_description: ptr::null(),
        };
        // SAFETY: `context` and `paths` are valid for the call (the stream copies both); `on_events` matches the
        // callback type; the `Inbox` it points to outlives the stream (see `Drop`).
        let stream = unsafe {
            FSEventStreamCreate(ptr::null(), on_events, &raw const context, paths.0.as_ptr(), since, 0.0, CREATE_FLAGS)
        };
        let stream = NonNull::new(stream).ok_or_else(|| io::Error::other("the stream could not be created"))?;
        // SAFETY: the label is a NUL-terminated static string; null attributes ask for a serial queue.
        let queue = unsafe { dispatch_queue_create(c"fresher.journal".as_ptr(), ptr::null()) };
        let Some(queue) = NonNull::new(queue) else {
            // SAFETY: the stream was created above, never started, and is released once.
            unsafe { FSEventStreamRelease(stream.as_ptr()) };
            return Err(io::Error::other("the queue could not be created"));
        };
        let stream = Self { raw: stream, queue, inbox };
        // SAFETY: both references are live; the stream is not started yet.
        unsafe { FSEventStreamSetDispatchQueue(stream.raw.as_ptr(), stream.queue.as_ptr()) };
        // SAFETY: the stream is live and scheduled on its queue.
        if unsafe { FSEventStreamStart(stream.raw.as_ptr()) } == 0 {
            return Err(io::Error::other("the stream could not be started"));
        }
        Ok(stream)
    }

    /// Asks the system to deliver every event it holds for the stream, and waits until it has.
    pub(crate) fn flush(&self) {
        // SAFETY: the stream is live and started.
        unsafe { FSEventStreamFlushSync(self.raw.as_ptr()) };
    }

    pub(crate) fn inbox(&self) -> &Inbox {
        &self.inbox
    }
}

impl Drop for Stream {
    fn drop(&mut self) {
        // SAFETY: the stream is live; stopping a stream that is not started does nothing.
        unsafe { FSEventStreamStop(self.raw.as_ptr()) };
        // SAFETY: the stream is live; after this no new callback is scheduled.
        unsafe { FSEventStreamInvalidate(self.raw.as_ptr()) };
        // SAFETY: the queue is live and serial: this returns after every callback already on it has run, so none
        // runs once the `Inbox` is dropped.
        unsafe { dispatch_sync_f(self.queue.as_ptr(), ptr::null_mut(), nothing) };
        // SAFETY: the stream is invalidated and released once, here.
        unsafe { FSEventStreamRelease(self.raw.as_ptr()) };
        // SAFETY: the queue was created by us and is released once, here.
        unsafe { dispatch_release(self.queue.as_ptr()) };
    }
}

/// A `CoreFoundation` object we own, released when dropped.
struct CfOwned(NonNull<c_void>);

impl CfOwned {
    fn new(object: CfTypeRef) -> Option<Self> {
        NonNull::new(object.cast_mut()).map(Self)
    }
}

impl Drop for CfOwned {
    fn drop(&mut self) {
        // SAFETY: we own one reference to the object, released once, here.
        unsafe { CFRelease(self.0.as_ptr()) };
    }
}

/// A `CFArray` of `CFString`s made from `paths`; the strings are owned by the array.
fn path_array(paths: &[CString]) -> io::Result<CfOwned> {
    let strings = paths
        .iter()
        .map(|path| {
            // SAFETY: `path` is NUL-terminated; the result is a new string we own, or null.
            CfOwned::new(unsafe { CFStringCreateWithFileSystemRepresentation(ptr::null(), path.as_ptr()) })
                .ok_or_else(|| io::Error::from(io::ErrorKind::InvalidInput))
        })
        .collect::<io::Result<Vec<_>>>()?;
    let values: Vec<CfTypeRef> = strings.iter().map(|string| string.0.as_ptr().cast_const()).collect();
    let count = CfIndex::try_from(values.len()).map_err(|_| io::Error::from(io::ErrorKind::InvalidInput))?;
    // SAFETY: `values` holds `count` live strings; the array retains each through the standard callbacks, so the
    // strings' own references can be released after it.
    let array = unsafe { CFArrayCreate(ptr::null(), values.as_ptr(), count, &raw const kCFTypeArrayCallBacks) };
    CfOwned::new(array).ok_or_else(|| io::Error::other("the path array could not be created"))
}
