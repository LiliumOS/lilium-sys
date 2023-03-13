use core::ffi::{c_void, c_long};
use crate::uuid::Uuid;

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


#[allow(improper_ctypes)]
extern "C"{
    pub fn IORead(hdl: HandlePtr<IOHandle>, buf: *mut c_void,len: usize) -> SysResult;
    pub fn IOWrite(hdl: HandlePtr<IOHandle>, buf: *const c_void, len: usize) -> SysResult;

    /// 
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


    pub fn OpenCharacterDevice(hdl: *mut HandlePtr<IOHandle>, devid: Uuid) -> SysResult;
    pub fn OpenBlockDevice(hdl: *mut HandlePtr<IOHandle>, devid: Uuid) -> SysResult;
    pub fn OpenLegacyCharacterDevice(hdl: *mut HandlePtr<IOHandle>, maj: u32, min: u32) -> SysResult;
    pub fn OpenLegacyBlockDevice(hdl: *mut HandlePtr<IOHandle>, maj: u32, min: u32) -> SysResult;

    pub fn CreatePipe(write_hdl: *mut HandlePtr<IOHandle>, read_hdl: *mut HandlePtr<IOHandle>,mode: u32, buffer_size: c_long) -> SysResult;
}