use core::ffi::{c_int, c_long, c_ulong, c_void};

use super::{
    handle::*,
    kstr::{KStrCPtr, KStrPtr},
    result::SysResult,
    time::Duration,
};

#[repr(transparent)]
pub struct ThreadHandle(Handle);

#[repr(C)]
pub struct ThreadStartContext {
    pub th_stack: *mut c_void,
    pub th_interal: *mut c_void,
    pub th_tlsbase: *mut c_void,
    pub th_start: extern "C" fn(*mut c_void, HandlePtr<ThreadHandle>, *mut c_void),
    #[doc(hidden)]
    pub __private: (),
}

#[allow(improper_ctypes)]
extern "C" {
    pub fn StartThread(
        tsc: *const ThreadStartContext,
        thout: *mut HandlePtr<ThreadHandle>,
    ) -> SysResult;
    pub fn ParkThread() -> SysResult;
    pub fn UnparkThread(th: HandlePtr<ThreadHandle>) -> SysResult;
    pub fn YieldThread();
    pub fn AwaitAddress(addr: *mut c_void) -> SysResult;
    pub fn NotifyOne(addr: *mut c_void) -> SysResult;
    pub fn NotifyAll(addr: *mut c_void) -> SysResult;
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

    pub fn SetThreadName(th: HandlePtr<ThreadHandle>, name: KStrCPtr) -> SysResult;
    pub fn GetThreadName(th: HandlePtr<ThreadHandle>, name: KStrPtr) -> SysResult;

    pub fn tls_register_destructor(dtor: fn(*mut c_void), key: isize) -> SysResult;

    pub fn get_tls_block_size() -> c_ulong;
    /// Returns the offset from the beginning of the TLS base address for dyanmically allocated thread locals (via tss_t or pthread_key_t)
    pub fn get_tls_slide_offset() -> c_long;

    pub fn thread_init_after_start(th: HandlePtr<ThreadHandle>) -> SysResult;
    pub fn thread_init_self() -> SysResult;

    pub fn thread_register_after_start(
        cb: fn(*mut c_void, HandlePtr<ThreadHandle>) -> SysResult,
        udata: *mut c_void,
    ) -> SysResult;
    pub fn thread_register_init_self(
        cb: fn(*mut c_void) -> SysResult,
        udata: *mut c_void,
    ) -> SysResult;

    pub fn tls_alloc_dyn(size: usize) -> isize;
    pub fn tls_alloc_dyn_aligned(size: usize, align: usize) -> isize;
    pub fn tls_free_dyn(key: isize);
}
