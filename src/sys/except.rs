use core::ffi::{c_ulong, c_void};
use std::thread::Thread;

use crate::uuid::Uuid;

use super::{
    handle::Handle, kstr::KCSlice, option::ExtendedOptionHead, result::NonZeroSysResult,
    thread::ThreadHandle,
};

#[repr(transparent)]
pub struct ExceptionContextHandle(Handle);

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ExceptionInfo {
    pub except_code: Uuid,
    pub except_sys_used: *mut c_void,
    pub except_data_ref: *mut c_void,
    pub last_exception_info: *const ExceptionInfo,
    pub chain_buffer: *const c_ulong,
    pub trigger_code_addr: *mut c_void,
    pub trigger_code_stack_head: *mut c_void,
}

/// An option for opening the file
#[repr(C, align(32))]
#[derive(Copy, Clone)]
pub struct UnknownExceptHandlerOption {
    /// The header
    pub head: ExtendedOptionHead,
    /// The tail
    pub tail: [MaybeUninit<u8>; 64],
}

#[repr(C, align(32))]
pub union ExceptHandlerOption {
    /// The Header: Must be present on all subfields
    pub head: ExtendedOptionHead,
    /// Fallback type for all fields
    pub unknown: UnknownExceptHandlerOption,
}

/// An option for opening the file
#[repr(C, align(32))]
#[derive(Copy, Clone)]
pub struct ExceptHandlerOptionSetStack {
    /// The header
    pub head: ExtendedOptionHead,
    /// The stack pointer to set when execution the exception handler
    pub stack_base_addr: *mut c_void,
}

pub type ExceptHandler =
    unsafe extern "system" fn(*mut ExceptionInfo, HandlePtr<ExceptionContextHandle>) -> !;

#[allow(improper_ctypes)]
extern "system" {
    /// Installs or removes the kernel exception handler
    pub fn ExceptInstallHandler(
        except_handler: Option<ExceptHandler>,
        opts: *const KCSlice<ExceptHandlerOption>,
    ) -> SysResult;
    pub fn ExceptHandleSynchronous(ptr: *const ExceptInfo) -> SysResult;
    pub fn ExceptRaiseAsynchronous(
        hdl: HandlePtr<ThreadHandle>,
        ptr: *const ExceptInfo,
    ) -> SysResult;

    /// Load all registers from the given context (which is then released) and resumes execution at `code_addr`
    pub fn ExceptResumeAt(
        code_addr: *mut c_void,
        ctx: HandlePtr<ExceptionContextHandle>,
    ) -> NonZeroSysResult;

    /// Sets a register in the context. The value is the same size as a machine word
    /// regno is the same as for [`DebugWriteRegister`][super::debug::DebugWriteRegister]. Debug and Task registers cannot be modified.
    /// 
    /// Despite the name, regno is not limited to registers considered "General Purpose" on the architecture. Rather, GPR refers to the size of the value (at most the word size),
    /// as thevalue of the register is passed inline.
    pub fn ExceptSetGPR(
        ctx: HandlePtr<ExceptionContextHandle>,
        regno: u32,
        value: c_ulong,
    ) -> SysResult;

    /// Sets a register in the context to the value of a pointer. The value is the same size as a machine word
    /// regno is the same as for [`DebugWriteRegister`][super::debug::DebugWriteRegister]. Debug and Task registers cannot be modified.
    /// 
    /// `value` is the value itself, represented as a pointer. It is not read or written to by this function. 
    /// [`INVALID_MEMORY`][crate::sys::result::errors::INVALID_MEMORY] is not returned by this function.
    pub fn ExceptSetPointerReg(
        ctx: HandlePtr<ExceptionContextHandle>,
        regno: u32,
        value: *mut c_void,
    ) -> SysResult;

    /// Sets a register in the context.
    /// regno is the same as for [`DebugWriteRegister`][super::debug::DebugWriteRegister]. Debug and Task registers cannot be modified.
    /// `size` is the size of `value` in bytes, `value` points to that many bytes to read the register from. Valid sizes depend on `regno` and the architecture
    pub fn ExceptSetRegister(
        ctx: HandlePtr<ExceptionContextHandle>,
        regno: u32,
        value: *const c_void,
        size: c_ulong,
    ) -> SysResult;

    /// Sets a register in the context.
    /// regno is the same as for [`DebugWriteRegister`][super::debug::DebugWriteRegister]. Debug and Task registers cannot be modified.
    /// `size` is the size of `value` in bytes, `value` points to that many bytes to read the register from. Valid sizes depend on `regno` and the architecture
    pub fn ExceptGetRegister(
        ctx: HandlePtr<ExceptionContextHandle>,
        regno: u32,
        value: *mut c_void,
        size: c_ulong,
    ) -> SysResult;
}
