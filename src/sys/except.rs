use core::{
    ffi::{c_ulong, c_void},
    mem::MaybeUninit,
};

use crate::uuid::Uuid;

use super::{
    handle::{Handle, HandlePtr},
    kstr::KCSlice,
    option::ExtendedOptionHead,
    result::{NonZeroSysResult, SysResult},
    thread::ThreadHandle,
};

#[repr(transparent)]
pub struct ExceptionContextHandle(Handle);

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "bytemuck", derive(bytemuck::Zeroable, bytemuck::Pod))]
pub struct ExceptionStatusInfo {
    pub except_code: Uuid,
    pub except_info: u64,
    pub except_reason: u64,
}

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct ExceptionInfo {
    pub status: ExceptionStatusInfo,
    pub except_sys_used: *mut c_void,
    pub last_exception_info: *const ExceptionInfo,
    pub except_data_ref: *mut c_void,
    pub except_data_size: usize,
    pub trigger_code_addr: *mut c_void,
    pub trigger_code_stack_head: *mut c_void,
}

/// An option for opening the file
#[repr(C, align(32))]
#[derive(Copy, Clone)]
#[cfg_attr(feature = "bytemuck", derive(bytemuck::Zeroable))]
pub struct UnknownExceptHandlerOption {
    /// The header
    pub head: ExtendedOptionHead,
    /// The tail
    pub tail: [MaybeUninit<u8>; 64],
}

#[cfg(feature = "bytemuck")]
unsafe impl bytemuck::AnyBitPattern for UnknownExceptHandlerOption {}

#[repr(C, align(32))]
#[cfg_attr(feature = "bytemuck", derive(bytemuck::AnyBitPattern))]
#[derive(Copy, Clone)]
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

/// Return value from a registered exception hook, that indicates that t
pub const EXCEPT_HOOK_CONTINUE: isize = 0;

pub const EXCEPT_HOOK_RESUME: isize = 1;
/// Return value from a registered exception hook, that indicates that further hooks should be exceuted, but after resolving the last hook (that does not return [`EXCEPT_HOOK_ABORT`]),
///  resume from the
pub const EXCEPT_HOOK_OK: isize = 2;
pub const EXCEPT_HOOK_ABORT: isize = -1;

#[cfg(feature = "base")]
unsafe extern "system" {

    /// Aborts the calling thread by reporting  `except` as having been recieved but not handled
    ///
    ///
    /// ## Default Behaviour
    ///
    /// Except for certain exceptions, which are specially recognized by the kernel to do something else,
    /// If no [`ExceptHandler`] is installed (or calling the [`ExceptHandler`] triggers an exception) when a thread recieves an exception,
    ///  the thread is terminated in the same manner as calling [`UnmanagedException`].
    #[cold]
    pub unsafe fn UnmanagedException(except: *const ExceptionStatusInfo) -> !;

    /// Installs or removes the kernel exception handler
    pub unsafe fn ExceptInstallHandler(
        except_handler: Option<ExceptHandler>,
        opts: *const KCSlice<ExceptHandlerOption>,
    ) -> SysResult;
    pub fn ExceptHandleSynchronous(
        ptr: *const ExceptionStatusInfo,
        data: *const c_void,
    ) -> SysResult;
    pub fn ExceptRaiseAsynchronous(
        hdl: HandlePtr<ThreadHandle>,
        ptr: *const ExceptionStatusInfo,
        data: *const c_void,
        data_sz: usize,
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

    pub fn ExceptGetStopAddr(
        ctx: HandlePtr<ExceptionContextHandle>,
        value: *mut *mut c_void,
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

    /// Registers a function to run when the standard USI handler begins.
    /// If any hook calls causes a synchronous exception (unless it installs a non-USI exception handler), the process calls [`UnmanagedException`] immediately.
    /// Hooks are entered in the order they are registered.
    ///
    /// However, it should be noted that asynchronous exceptions will enter the hooks, so
    pub fn except_hook(
        userdata: *mut c_void,
        hook: unsafe extern "system" fn(*mut c_void, *const ExceptionInfo) -> isize,
    ) -> SysResult;
}
