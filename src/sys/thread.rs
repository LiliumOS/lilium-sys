use core::{
    ffi::{c_int, c_long, c_ulong, c_void},
    mem::MaybeUninit,
};

use super::{
    except::{ExceptionInfo, ExceptionStatusInfo},
    handle::*,
    kstr::{KCSlice, KStrCPtr, KStrPtr},
    option::ExtendedOptionHead,
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
    pub th_start: extern "system" fn(*mut c_void, HandlePtr<ThreadHandle>, *mut c_void),
    #[doc(hidden)]
    pub __private: (),
}

#[derive(Copy, Clone)]
#[repr(C, align(32))]
#[cfg_attr(feature = "bytemuck", derive(bytemuck::Zeroable))]
pub struct ThreadStartOptionRaw {
    pub head: ExtendedOptionHead,
    pub payload: [MaybeUninit<u8>; 64],
}

#[cfg(feature = "bytemuck")]
unsafe impl bytemuck::AnyBitPattern for ThreadStartOptionRaw {}

#[derive(Copy, Clone)]
#[repr(C, align(32))]
#[cfg_attr(feature = "bytemuck", derive(bytemuck::AnyBitPattern))]
pub union ThreadStartOption {
    pub head: ExtendedOptionHead,
    pub raw: ThreadStartOptionRaw,
}

#[repr(C)]
#[derive(Copy, Clone)]
#[cfg_attr(feature = "bytemuck", derive(bytemuck::Zeroable, bytemuck::Pod))]
pub struct JoinStatusExit {
    /// The discriminant for the status. Always set to `0` for a [`JoinStatusExit`] (else non-zero)
    /// This is also specifically set to all ones bits
    pub discriminant: u64,
    pub exit_code: c_ulong,
    #[doc(hidden)]
    pub __reserved: [c_ulong; (size_of::<u64>() - size_of::<c_ulong>()) / size_of::<c_ulong>()],
}

#[repr(C)]
#[derive(Copy, Clone)]
#[cfg_attr(feature = "bytemuck", derive(bytemuck::AnyBitPattern))]
pub union JoinStatus {
    pub exit_code: JoinStatusExit,
    pub exit_exception: ExceptionStatusInfo,
}

def_option_type! {
    pub struct EventJoinThread("d898a750-04c4-5479-aa6d-6103d202ce28") {
        pub target_thread: HandlePtr<ThreadHandle>,
        pub status_or_exception: *mut JoinStatus,
    }
}

#[cfg(any(feature = "thread", doc))]
unsafe extern "system" {
    pub fn StartThread(
        tsc: *const ThreadStartContext,
        thout: *mut HandlePtr<ThreadHandle>,
        options: KCSlice<ThreadStartOption>,
    ) -> SysResult;
    pub safe fn YieldThread();

    pub fn ThreadExit(thr: c_int) -> !;
    pub fn GetCurrentThread() -> HandlePtr<ThreadHandle>;
    pub fn GetTLSBaseAddr(th: HandlePtr<ThreadHandle>, addrout: *mut *mut c_void) -> SysResult;
    pub fn SetTLSBaseAddr(th: HandlePtr<ThreadHandle>, addr: *mut c_void) -> SysResult;
    pub fn JoinThread(th: HandlePtr<ThreadHandle>, status: *mut JoinStatus) -> SysResult;
    pub fn DetachThread(th: HandlePtr<ThreadHandle>) -> SysResult;

    pub fn SendHandle(toth: HandlePtr<ThreadHandle>, sendhdl: HandlePtr<Handle>) -> SysResult;
    pub fn RecieveHandle(out: *mut HandlePtr<Handle>) -> SysResult;

    pub fn SetThreadName(th: HandlePtr<ThreadHandle>, name: KStrCPtr) -> SysResult;
    pub fn GetThreadName(th: HandlePtr<ThreadHandle>, name: *mut KStrPtr) -> SysResult;

    /// Sets the current thread to be the "control" thread of the current process.
    /// When the control thread exits or is killed by an exception, each other
    pub safe fn ControlProcessExit() -> SysResult;
    pub safe fn ReleaseProcessExit() -> SysResult;

    pub fn tls_register_destructor(dtor: fn(*mut c_void), key: isize) -> SysResult;

    pub fn get_tls_block_size() -> c_ulong;
    /// Returns the offset from the beginning of the TLS base address for dyanmically allocated thread locals (via tss_t or pthread_key_t)
    pub fn get_tls_slide_offset() -> c_long;

    pub unsafe fn thread_init_after_start(th: HandlePtr<ThreadHandle>) -> SysResult;
    pub unsafe fn thread_init_self() -> SysResult;

    pub unsafe fn thread_register_after_start(
        cb: fn(*mut c_void, HandlePtr<ThreadHandle>) -> SysResult,
        udata: *mut c_void,
    ) -> SysResult;
    pub unsafe fn thread_register_init_self(
        cb: fn(*mut c_void) -> SysResult,
        udata: *mut c_void,
    ) -> SysResult;

    pub unsafe fn tls_alloc_dyn(size: usize) -> isize;
    pub unsafe fn tls_alloc_dyn_aligned(size: usize, align: usize) -> isize;
    pub unsafe fn tls_free_dyn(key: isize);
}
