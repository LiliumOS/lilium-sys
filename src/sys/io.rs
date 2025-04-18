use crate::uuid::Uuid;
use core::ffi::{c_long, c_ulong, c_void};

use super::{handle::*, result::SysResult};

#[repr(transparent)]
pub struct IOHandle(Handle);

/// Places the thread in `BLOCKED` state if any operation on the handle (including the `OpenFile` operation) cannot complete immediately.
///
/// Blocking operations that do not complete immediately act like any other blocking syscall from the thread api
pub const MODE_BLOCKING: u32 = 0x00;

/// Returns immediately with WOULDBLOCK if any operation on the handle (including the `OpenFile` operation) cannot complete immediately
pub const MODE_NONBLOCKING: u32 = 0x01;

/// Returns immediately with PENDING if any operation on the handle (including the `OpenFile` operation) cannot complete immediaetly.
///
/// The operation is queued and performed in the background.
pub const MODE_ASYNC: u32 = 0x02;

pub const NOTIFY_INTERRUPT: u32 = 0x40;
pub const NOTIFY_SIGNAL_MASK: u32 = 0x3f;

pub const CHAR_READABLE: u32 = 0x01;
pub const CHAR_WRITABLE: u32 = 0x02;
pub const CHAR_SEEKABLE: u32 = 0x04;
pub const CHAR_RANDOMACCESS: u32 = 0x08;

pub const SEEK_FROM_START: u32 = 0;
pub const SEEK_FROM_END: u32 = 1;
pub const SEEK_FROM_CURRENT: u32 = 2;

#[repr(C)]
pub struct PollInfo {
    pub hdl: HandlePtr<IOHandle>,
    pub read_bytes: c_ulong,
    pub status: SysResult,
}

#[cfg(any(feature = "io", doc))]
unsafe extern "system" {

    /// Thread Local handle that is initialized to the standard input stream by the standard library
    #[thread_local]
    pub safe static __HANDLE_IO_STDIN: HandlePtr<IOHandle>;

    /// Thread Local handle that is initialized to the standard output stream by the standard library
    #[thread_local]
    pub safe static __HANDLE_IO_STDOUT: HandlePtr<IOHandle>;

    /// Thread Local handle that is initialized to the standard error stream by the standard library
    #[thread_local]
    pub safe static __HANDLE_IO_STDERR: HandlePtr<IOHandle>;

    /// Reads up to `len` bytes from the given
    pub fn IORead(hdl: HandlePtr<IOHandle>, buf: *mut c_void, len: c_ulong) -> SysResult;
    pub fn IOWrite(hdl: HandlePtr<IOHandle>, buf: *const c_void, len: c_ulong) -> SysResult;
    pub fn IOSeek(hdl: HandlePtr<IOHandle>, from: u32, offset: i64) -> SysResult;
    pub fn IOSeekFar(
        hdl: HandlePtr<IOHandle>,
        from: u32,
        offset: i128,
        abs_off: *mut u128,
    ) -> SysResult;

    /// Copies a number of bytes from `src_hdl` to `dest_hdl`, without an intermediate return to userspace
    /// This is intended to bemore efficient than performing individual `IORead` and `IOWrite` calls.
    ///
    /// Blocking Behaviour:
    /// * If either handle is `MODE_NONBLOCKING` and either operation would block, then `WOULDBLOCK` is returned
    /// * If both `src_hdl` and `dest_hdl` are configured `MODE_ASYNC` and either operation would block, then PENDING is returned, and the behaviour follows standard async I/O rules
    /// * Otherwise, the syscall blocks.
    pub fn IOCopy(
        src_hdl: HandlePtr<IOHandle>,
        dest_hdl: HandlePtr<IOHandle>,
        len: c_ulong,
    ) -> SysResult;

    /// Copies all available data from src_hdl to dest_hdl
    /// This is the same as an `IOCopy` between the handles with an arbitrarily large length, except:
    /// * If both src and dest are pipes, if dest has a larger or same-size buffer than src, then the operation is atomic
    /// * If dest is a socket, and either src is a datagram socket or a pipe, then the next block that is available (up to the minimum of the src buffer size and the dest packet size)
    ///    is written as a single unit (this is important if `dest` is a datagram socket)
    /// * Likewise, if `dest` is a pipe, and src is a datagram socket, the next datagram recieved, up to the dest buffer size, is written as a single unit.
    ///    If the datagram exceeds the buffer size, it is split.
    ///
    /// The length of the operation transfered is stored in `size`.
    pub fn IOCopyFull(
        src_hdl: HandlePtr<IOHandle>,
        dest_hdl: HandlePtr<IOHandle>,
        size: *mut u128,
    ) -> SysResult;

    /// Reads from the stream backed by `hdl` into `buf`, starting from `file_base`.
    /// `hdl` must be [`CHAR_SEEKABLE`].
    /// If `hdl` is [`CHAR_RANDOMACCESS`], the seek position is not modified by this syscall. Otherwise the seek position is unspecified (after the syscall)
    ///
    /// Returns the number of bytes read, or `0` if `file_base` is out of bounds for the file
    /// ## Errors
    /// Same errors as [`IORead`] and:
    /// * if `hdl` is not [`CHAR_SEEKABLE`] returns `UNSUPPORTED_OPERATION`
    pub fn IOReadRA(
        hdl: HandlePtr<IOHandle>,
        buf: *mut c_void,
        len: c_ulong,
        file_base: u64,
    ) -> SysResult;
    /// Writes to the stream backed by `hdl` from `buf`, starting from `file_base`.
    /// `hdl` must be [`CHAR_SEEKABLE`].
    /// If `hdl` is [`CHAR_RANDOMACCESS`], the seek position is not modified by this syscall. Otherwise the seek position is unspecified (after the syscall)
    ///
    /// Returns the number of bytes read, or `0` if `file_base` is out of bounds for the file
    /// ## Errors
    /// Same errors as [`IORead`] and:
    /// * if `hdl` is not [`CHAR_SEEKABLE`] returns `UNSUPPORTED_OPERATION`
    pub fn IOWriteRA(
        hdl: HandlePtr<IOHandle>,
        buf: *const c_void,
        len: c_ulong,
        file_base: u64,
    ) -> SysResult;

    pub fn GetIOCharacteristics(hdl: HandlePtr<IOHandle>) -> SysResult;

    pub fn SetIOBlockingMode(hdl: HandlePtr<IOHandle>, mode: u32) -> SysResult;
    pub fn SetIONotifyMode(hdl: HandlePtr<IOHandle>, notif_flags: u32) -> SysResult;
    pub fn SetIONotifyAddr(hdl: HandlePtr<IOHandle>, addr: *mut c_void) -> SysResult;

    pub fn IOPoll(hdl: HandlePtr<IOHandle>, read_len: *mut c_ulong) -> SysResult;
    pub fn IOPollAll(poll_array: *mut PollInfo, poll_array_len: c_ulong) -> SysResult;
    pub fn IOAbort(hdl: HandlePtr<IOHandle>) -> SysResult;
    pub fn IOJoin(hdl: HandlePtr<IOHandle>) -> SysResult;
    pub fn IOJoinAll(join_array: *mut PollInfo, join_array_len: c_ulong) -> SysResult;
    pub fn IOPause(hdl: HandlePtr<IOHandle>) -> SysResult;
    pub fn IOResume(hdl: HandlePtr<IOHandle>) -> SysResult;
    /// When an async I/O Operation completes on `hdl`, atomically writes the length to `len` and notifies it as though by [`NotifyAddress`][crate::sys::thread::NotifyAddress].
    /// len must obey the constraints set by `NotifyAddress`
    /// The `notify_mask` is set to `0` (notifies all threads)
    /// # Errors
    /// If `hdl` is not a valid handle, returns `INVALID_HANDLE``.
    ///
    /// Returns an error if `len` would be invalid for `NotifyAddress`
    pub unsafe fn IONotify(hdl: HandlePtr<IOHandle>, len: *mut usize) -> SysResult;

    /// Restarts a blocking I/O Operation that was interupted or timed out.
    pub fn IORestart(hdl: HandlePtr<IOHandle>) -> SysResult;

    pub fn OpenLegacyCharDevice(hdl: *mut HandlePtr<IOHandle>, maj: u32, min: u32) -> SysResult;
    pub fn OpenLegacyBlockDevice(hdl: *mut HandlePtr<IOHandle>, maj: u32, min: u32) -> SysResult;

    pub fn CreatePipe(
        write_hdl: *mut HandlePtr<IOHandle>,
        read_hdl: *mut HandlePtr<IOHandle>,
        mode: u32,
        buffer_size: c_long,
    ) -> SysResult;

    /// Creates an `IOHandle` that accesses memory owned by a process.
    pub fn CreateMemoryBuffer(
        hdl: *mut HandlePtr<IOHandle>,
        mode: u32,
        buf: *mut c_void,
        len: c_ulong,
        chars: u32,
    ) -> SysResult;

    /// Closes an IO stream open in the given `IOHandle`.
    ///
    /// ## Errors
    ///
    /// Returns `INVALID_HANDLE` if `hdl` is not a valid handle.
    pub fn CloseIOStream(hdl: HandlePtr<IOHandle>) -> SysResult;

    /// Opens a new IO Handlde that has the same properties and refers to the same object as an existing one, except it can only peform such operations as described by `char_mask`
    /// The `out_hdl` has the same type as `in_hdl` (if it is a File, `out_hdl` will also refer to the same file directly). Permission checks are not performed when creating the new file descriptor.
    ///
    /// The characteristics of the output file are determined by bitwise and of the Characteristics mask and the characteristics of the given handle -
    ///  `char_mask` can be used to disable operations, but not add new ones.
    ///
    /// ## Errors
    ///
    /// Returns `INVALID_HANDLE` if `in_hdl` is not a valid handle.
    ///
    /// Returns `RESOURCE_LIMIT_EXHAUSTED` if the total number of handles the thread has open exceeds the handle limit.
    pub fn DuplicateIOHandle(
        out_hdl: *mut HandlePtr<IOHandle>,
        in_hdl: HandlePtr<IOHandle>,
        char_mask: u32,
    ) -> SysResult;

    /// Sets the I/O buffer size for partially complete I/O operations.
    ///
    /// When an IO operation is requested, that is not completely fulfilled, and partial data is available,
    /// the operation will block until at least the specified number of bytes are available.
    ///
    /// If the data available is final, it may return less than this size,
    ///  and if fewer bytes are requested, only that many bytes will be required before returning.
    ///
    /// This function is guaranteed to affect:
    /// * The Read end of pipes
    /// * A socket
    /// * An IPC Connection
    ///
    /// And this function may affect a device handle referring to a character device. It is device specific whether it affects handles to that device
    ///
    pub fn IOSetMinReadSize(hdl: HandlePtr<IOHandle>, buf: c_ulong) -> SysResult;
}
