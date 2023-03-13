use core::ffi::{c_void, c_int};

use super::{handle::*, result::SysResult, time::Duration, kstr::{KStrCPtr, KStrPtr}};

#[repr(transparent)]
pub struct ThreadHandle(Handle);

#[repr(C)]
pub struct ThreadStartContext{
    pub th_stack: *mut c_void,
    pub th_interal: *mut c_void,
    pub th_tlsbase: *mut c_void,
    pub th_start: extern "C" fn(*mut c_void,HandlePtr<ThreadHandle>,*mut c_void),
    #[doc(hidden)]
    pub __private: (),
}

#[allow(improper_ctypes)]
extern "C"{
    pub fn StartThread(tsc: *const ThreadStartContext, thout: *mut HandlePtr<ThreadHandle>) -> SysResult;
    pub fn ParkThread() -> SysResult;
    pub fn UnparkThread(th: HandlePtr<ThreadHandle>) -> SysResult;
    pub fn YieldThread();
    pub fn AwaitAddress(addr: *mut c_void) -> SysResult;
    pub fn SignalOne(addr: *mut c_void) -> SysResult;
    pub fn SignalAll(addr: *mut c_void) -> SysResult;
    pub fn SetBlockingTimeout(dur: *const Duration);
    pub fn SleepThread(dur: *const Duration) -> SysResult;
    pub fn PauseThread() -> SysResult;
    pub fn InterruptThread(th: HandlePtr<ThreadHandle>) -> SysResult;
    pub fn Interrupted() -> SysResult;
    pub fn ClearBlockingTimeout();
    pub fn ThreadExit(thr: c_int) -> !;
    pub fn GetCurrentThread() -> HandlePtr<ThreadHandle>;
    pub fn GetTLSBaseAddr(th: HandlePtr<ThreadHandle>, addrout: *mut *mut c_void) -> SysResult;
    pub fn SetTLSBaseAddr(th: HandlePtr<ThreadHandle>, addr: *mut c_void) -> SysResult;
    pub fn JoinThread(th: HandlePtr<ThreadHandle>) -> SysResult;
    pub fn DetachThread(th: HandlePtr<ThreadHandle>) -> SysResult;

    pub fn SendHandle(toth: HandlePtr<ThreadHandle>, sendhdl: HandlePtr<Handle>) -> SysResult;
    pub fn RecieveHandle(out: *mut HandlePtr<Handle>) -> SysResult;

    pub fn SetThreadName(th: HandlePtr<ThreadHandle>, name: KStrCPtr)-> SysResult;
    pub fn GetThreadName(th: HandlePtr<ThreadHandle>, name: KStrPtr) -> SysResult;
}