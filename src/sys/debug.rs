use core::ffi::c_void;

use super::{handle::{Handle,HandlePtr}, thread::ThreadHandle, result::SysResult,signal::{SignalSet,SignalInformation}};


#[repr(transparent)]
pub struct DebugHandle(Handle);

#[allow(improper_ctypes)]
extern "C"{
    pub fn DebugAttach(th: HandlePtr<ThreadHandle>, dhout: *mut HandlePtr<DebugHandle>) -> SysResult;
    pub fn DebugDetach(dh: HandlePtr<DebugHandle>) -> SysResult;
    pub fn DebugSuspend(dh: HandlePtr<DebugHandle>) -> SysResult;
    pub fn DebugSuspendAll(dh: HandlePtr<DebugHandle>) -> SysResult;
    pub fn DebugReadMemory(dh: HandlePtr<DebugHandle>, addr: usize,  buf: *mut c_void, len: usize) -> SysResult;
    pub fn DebugWriteMemory(dh: HandlePtr<DebugHandle>, addr:usize, buf: *const c_void, len: usize) -> SysResult;
    pub fn DebugCaptureSignal(dh: HandlePtr<DebugHandle>, sig: SignalSet, info_buf: *mut SignalInformation) -> SysResult;
    pub fn DebugAwaitCapture(dh: HandlePtr<DebugHandle>) -> SysResult;
    pub fn DebugPollCapture(dh: HandlePtr<DebugHandle>) -> SysResult;
    pub fn DebugResume(dh: HandlePtr<DebugHandle>) -> SysResult;
}