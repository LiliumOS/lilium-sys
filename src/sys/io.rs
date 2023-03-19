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

pub const CHAR_READABLE: c_long = 0x01;
pub const CHAR_WRITABLE: c_long = 0x02;
pub const CHAR_SEEKABLE: c_long = 0x04;
pub const CHAR_RANDOMACCESS: c_long = 0x08;

pub const SEEK_FROM_START: u32 = 0;
pub const SEEK_FROM_END: u32 = 1;
pub const SEEK_FROM_CURRENT: u32 = 2;

#[allow(improper_ctypes)]
extern "C" {
    pub fn IORead(hdl: HandlePtr<IOHandle>, buf: *mut c_void, len: c_ulong) -> SysResult;
    pub fn IOWrite(hdl: HandlePtr<IOHandle>, buf: *const c_void, len: c_ulong) -> SysResult;
    pub fn IOSeek(hdl: HandlePtr<IOHandle>, from: u32, offset: c_long) -> SysResult;

    pub fn IOReadRA(
        hdl: HandlePtr<IOHandle>,
        buf: *mut c_void,
        len: c_ulong,
        file_base: c_ulong,
    ) -> SysResult;
    pub fn IOWriteRA(
        hdl: HandlePtr<IOHandle>,
        buf: *const c_void,
        len: c_ulong,
        file_base: c_ulong,
    ) -> SysResult;

    pub fn GetIOCharacteristics(hdl: HandlePtr<IOHandle>) -> SysResult;

    pub fn SetIOBlockingMode(hdl: HandlePtr<IOHandle>, mode: u32) -> SysResult;
    pub fn SetIONotifyMode(hdl: HandlePtr<IOHandle>, notif_flags: u32) -> SysResult;
    pub fn SetIONotifyAddr(hdl: HandlePtr<IOHandle>, addr: *mut c_void) -> SysResult;

    pub fn IOPoll(hdl: HandlePtr<IOHandle>) -> SysResult;
    pub fn IOAbort(hdl: HandlePtr<IOHandle>) -> SysResult;
    pub fn IOJoin(hdl: HandlePtr<IOHandle>) -> SysResult;
    pub fn IOPause(hdl: HandlePtr<IOHandle>) -> SysResult;
    pub fn IOResume(hdl: HandlePtr<IOHandle>) -> SysResult;

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

    pub fn CreateMemoryBuffer(
        hdl: *mut HandlePtr<IOHandle>,
        mode: u32,
        buf: *mut c_void,
        len: c_ulong,
        chars: u32,
    ) -> SysResult;
}
