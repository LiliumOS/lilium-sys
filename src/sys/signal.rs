use core::ffi::c_void;

use super::{
    handle::{Handle, HandlePtr},
    process::ProcessHandle,
    result::SysResult,
    thread::ThreadHandle,
};

#[repr(C)]
pub struct SignalSet(pub [u64; 2]);

#[repr(C)]
pub union SignalSourcePtr {
    pub source_handle: HandlePtr<Handle>,
    pub faulting_instr: *mut c_void,
}

#[repr(C)]
pub union SignalAuxSource {
    pub access_addr: *mut c_void,
    pub decoded_opcode: u64,
}

#[repr(C)]
pub struct SignalInformation {
    pub explicit_source_thread: HandlePtr<ThreadHandle>,
    pub source_ptr: SignalSourcePtr,
    pub auxilary_source: SignalAuxSource,
}

pub const SIGNAL_INTERRUPT_FLAG: u32 = 0x01;
pub const SIGNAL_GLOBAL_FLAG: u32 = 0x02;

#[allow(improper_ctypes)]
extern "C" {
    pub fn SignalThread(th: HandlePtr<ThreadHandle>, signo: u32) -> SysResult;
    pub fn SignalProcess(ph: HandlePtr<ProcessHandle>, signo: u32) -> SysResult;
    pub fn SetSignalHandlingThread(th: HandlePtr<ThreadHandle>) -> SysResult;
    pub fn SetSignalThreadFlags(flags: u32) -> SysResult;
    pub fn ClearSignalThreadFlags(flags: u32) -> SysResult;

    pub fn HandleSignals(
        sighdl: Option<unsafe extern "C" fn(u32, *mut SignalInformation)>,
        sigset: *const SignalSet,
    ) -> SysResult;
    pub fn BlockSignals(sigset: *const SignalSet) -> SysResult;
    pub fn ResetSignals(sigset: *const SignalSet) -> SysResult;
    pub fn LimitSignals(sigset: *const SignalSet) -> SysResult;
    pub fn PendingSignals(sigset: *mut SignalSet) -> SysResult;
}
