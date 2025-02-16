//! Debugging Interfaces for LiliumOS

use core::ffi::c_void;

use super::{
    except::ExceptionInfo,
    handle::{Handle, HandlePtr},
    kstr::KStrPtr,
    result::SysResult,
    thread::ThreadHandle,
};

/// A Handle that represents a debugging operation in progress.
#[repr(transparent)]
pub struct DebugHandle(Handle);

#[repr(C)]
#[cfg_attr(feature = "bytemuck", derive(bytemuck::Zeroable))]
#[derive(Copy, Clone)]
pub struct DebugMappingInfo {
    pub vaddr_lo: usize,
    pub vaddr_hi: usize,
    pub mapping_name: KStrPtr,
    pub kind_and_attrs: u32,
    pub page_status: u32,
    pub backing_paddr: u64,
}

#[cfg(any(feature = "debug", doc))]
#[expect(improper_ctypes)]
unsafe extern "system" {
    /// Attaches a debugger to the given thread, and returns a handle to that debugger.
    ///
    ///
    /// ## Errors
    ///
    /// If the kernel limit `DEBUG_OPERATIONS` is exceeded by the current thread, `RESOURCE_LIMIT_EXHAUSTED` is returned.
    ///
    /// If the thread permission `DEBUG_ATTACH` for `th` is denied to the current thread, `PERMISSION` is returned.
    ///
    /// If `th` is not a valid `ThreadHandle`, `INVALID_HANDLE` is returned.
    ///
    /// If allocating memory asociated with the returned handle fails, `RESOURCE_LIMIT_EXHAUSTED` is returned.
    ///
    /// If `dhout` does not point to appropriate writeable memory, `INVALID_MEMORY` is returned.
    ///
    /// ## Notes
    ///
    /// It is possible to attach to the current thread, but an explicit `ThreadHandle` is still required in this case.
    pub fn DebugAttach(
        th: HandlePtr<ThreadHandle>,
        dhout: *mut HandlePtr<DebugHandle>,
    ) -> SysResult;

    /// Detaches a debug handle from a process and closes the handle.
    ///
    /// ## Errors
    ///
    /// If `dh` is not a valid `DebugHandle`, `INVALID_HANDLE` is returned.
    pub fn DebugDetach(dh: HandlePtr<DebugHandle>) -> SysResult;
    /// Suspends the thread referred to by the debug handle
    ///
    /// The thread is guaranteed to be suspended at the latest when an asynchronous signal would be delivered to that thread.
    ///
    /// This syscall waits until the suspend is complete (this is not considered to be a blocking operation and is not subject to standard blocking syscall rules).
    ///
    /// ## Errors
    ///
    /// If `dh` is not a valid `DebugHandle`, `INVALID_HANDLE` is returned.
    ///
    /// If `dh` refers to the current thread, `WOULD_BLOCK` is returned. A thread cannot suspend itself.
    ///
    pub fn DebugSuspend(dh: HandlePtr<DebugHandle>) -> SysResult;
    /// Suspends all threads in the process referred to by the debug handle
    ///
    /// This syscall waits until the suspend is complete (this is not considered to be a blocking operation and is not subject to standard blocking syscall rules).
    ///
    /// ## Errors
    ///
    /// If `dh` is not a valid `DebugHandle`, `INVALID_HANDLE` is returned.
    ///
    /// If `dh` refers to a thread in the current process, `WOULD_BLOCK` is returned. A thread cannot suspend itself.
    ///
    /// If the `DEBUG_SUSPEND_ALL` permission is denied for the process this syscall affects to the current thread, `PERMISSON` is returned.
    ///
    /// ## Notes
    ///
    /// `DebugSuspendAll` may wait until all affected threads are suspended, or only the thread directly referred to by `dh` is suspended, which is unspecified.
    ///
    pub fn DebugSuspendAll(dh: HandlePtr<DebugHandle>) -> SysResult;

    /// Reads memory from the address space of the thread being debugged by `dh`.
    ///
    /// The thread must be suspended before this syscall is used.
    ///
    /// ## Errors
    ///
    /// If `dh` is not a valid `DebugHandle`, `INVALID_HANDLE` is returned.
    ///
    /// If `dh` is not suspended, `INVALID_STATE` is returned.
    ///
    /// If `buf` does not point to suitable storage writable by the current thread, `INVALID_MEMORY` is returned.
    ///
    /// If every byte in `[addr,addr+1en)` is not part of a mapping that is accessible to the thread referred to by `dh`, `DEBUG_TARGET_NOT_MAPPED` is returned.
    ///
    /// If any byte in `[addr,addr+1en)` is part of a mapping that is `MAP_KIND_SECURE` or `MAP_KIND_ENCRYPTED`, `PERMISSION` is returned.
    ///
    /// ## Notes
    ///
    /// Process and thread-private mappings may be accessed by this function. Only mappings visible to the target thread are accessed.
    ///  Other overlapping thread-private mappings are not accessed.
    ///
    pub fn DebugReadMemory(
        dh: HandlePtr<DebugHandle>,
        addr: usize,
        buf: *mut c_void,
        len: usize,
    ) -> SysResult;

    /// writes memory to the address space of the thread being debugged by `dh`.
    ///
    /// The thread must be suspended before this syscall is used.
    ///
    /// ## Errors
    ///
    /// If `dh` is not a valid `DebugHandle`, `INVALID_HANDLE` is returned.
    ///
    /// If `dh` is not suspended, `INVALID_STATE` is returned.
    ///
    /// If `buf` does not point to suitable storage readable by the current thread, `INVALID_MEMORY` is returned.
    ///
    /// If every byte in `[addr,addr+1en)` is not part of a mapping that is accessible to the thread referred to by `dh`, `INVALID_MEMORY` is returned.
    ///
    /// If any byte in `[addr,addr+1en)` is part of a mapping that is `MAP_KIND_SECURE` or `MAP_KIND_ENCRYPTED`, `PERMISSION` is returned.
    ///
    /// If every byte in `[addr,addr+1en)` is not part of a mapping that is accessible to the thread referred to by `dh`, `DEBUG_TARGET_NOT_MAPPED` is returned.
    ///
    /// ## Notes
    ///
    /// Process and thread-private mappings may be accessed by this function. Only mappings visible to the target thread are accessed.
    ///  Other overlapping thread-private mappings are not accessed.
    ///
    /// `DebugWriteMemory` can modify pages that are not mapped as writable (for example, to insert breakpoints into executable code).
    pub fn DebugWriteMemory(
        dh: HandlePtr<DebugHandle>,
        addr: usize,
        buf: *const c_void,
        len: usize,
    ) -> SysResult;

    /// Reads the register specifed by regno from the debugged thread.
    ///
    /// THe thread must be suspended before this syscall is used.
    ///
    /// `regno` is the architectural DWARF register number you wish to read from. Certain registers cannot be read.
    /// `meta` specifies additional information about the selected register, useful for disambiguating between different sizes of register
    ///
    /// ## Errors
    ///
    /// If `dh` is not a valid `DebugHandle`, `INVALID_HANDLE` is returned.
    ///
    /// If `dh` is not suspended, `INVALID_STATE` is returned.
    ///
    /// If `buf` does not point to suitable storage writable by the current thread, `INVALID_MEMORY` is returned.
    ///
    ///
    /// If `regno` is not a valid register, returns `INVALID_OPERATION`. If `regno` cannot be read on the current architecture, returns `INVALID_OPERATION`
    ///
    /// If `meta` is not valid for the specified register, returns `INVALID_OPERATION`.
    ///
    /// ## Notes
    ///
    /// The required size for `buf` depends on `regno`.
    ///
    /// ## Architecture Specific Notes
    ///
    /// ### x86_64
    ///
    /// The task register `tr`, and the ldt register `ldtr`, as well as control registers, and trace registers, cannot be read by this function.
    ///
    /// Additionally, the upper vector registers `xmm16` through `xmm31` are only available if an appropriate extension is available on the processor.
    ///
    /// For Vector registers, `meta` is the chosen type: `0` means xmm, `1` means ymm, and `2` means zmm. ymm and zmm registers are only available if an appropriate extension is available on the processor.
    ///
    /// General Purpose registers, rflags, `fs.base` and `gs.base` require 8 bytes to store. This is not configured by `meta`
    ///
    /// Segment registers can be read and require 8 bytes to store. This is not configured by `meta`
    ///
    /// Debug Registers may be accessed to allow a debugger to use hardware debugging. DR4 and DR5 are not defined. These require 8 bytes to store.
    /// Some bits of DR6 and DR7 may be masked by the kernel, including:
    /// * Bit 12 of both DR6 and DR7 are read as 0 by this function
    ///
    /// At the very least bits 0-3 and bit 14 of DR6, as well as bits 0-7 and bits 16-31 of DR7 may be read by this function
    ///
    /// When a debugger is attached to a process, DR0-DR3 are reset to 0. DR6 and DR7
    ///
    pub fn DebugReadRegister(
        dh: HandlePtr<DebugHandle>,
        regno: u32,
        buf: *mut c_void,
        meta: u32,
    ) -> SysResult;

    /// Writes the register specifed by regno from the debugged thread.
    ///
    /// THe thread must be suspended before this syscall is used.
    ///
    /// `regno` is the architectural DWARF register number you wish to write to. Certain registers cannot be written.
    /// `meta` specifies additional information about the selected register, useful for disambiguating between different sizes of register
    ///
    /// ## Errors
    ///
    /// If `dh` is not a valid `DebugHandle`, `INVALID_HANDLE` is returned.
    ///
    /// If `dh` is not suspended, `INVALID_STATE` is returned.
    ///
    /// If `buf` does not point to suitable storage writable by the current thread, `INVALID_MEMORY` is returned.
    ///
    /// If every byte in `[addr,addr+1)` is not part of a mapping that is accessible to the thread referred to by `dh`, `INVALID_MEMORY` is returned.
    ///
    /// If any byte in `[addr,addr+1)` is part of a mapping that is `MAP_KIND_SECURE` or `MAP_KIND_ENCRYPTED`, `PERMISSION` is returned.
    ///
    /// If `regno` is not a valid register, returns `INVALID_OPERATION`. If `regno` cannot be read on the current architecture, returns `INVALID_OPERATION`
    ///
    /// If `meta` is not valid for the specified register, returns `INVALID_OPERATION`.
    ///
    ///
    /// ## Notes
    ///
    /// The required size for `buf` depends on `regno`.
    ///
    /// ## Architecture Specific Notes
    ///
    /// ### x86_64
    ///
    /// The task register `tr`, and the ldt register `ldtr`, as well as control registers and trace registers, cannot be accessed by this function.
    ///
    /// Further, the segment registers cannot be modified (but `fs.base` and `gs.base` can be).
    ///
    /// Additionally, the upper vector registers `xmm16` through `xmm31` are only available if an appropriate extension is available on the processor.
    ///
    /// For Vector registers, `meta` is the chosen type: `0` means xmm, `1` means ymm, and `2` means zmm. The chosen size is only available if an appropriate extension is avalable on the processor.
    ///
    ///
    /// General Purpose registers, rflags, `fs.base` and `gs.base` require 8 bytes to load. This is not configured by `meta`
    ///
    /// Writes to `rflags` ignore modifications to any reserved bits, and to the following bits:
    /// * Bit 9 (IE)
    /// * Bits 12-13 (IOPL)
    /// * Bit 14 (NT)
    /// * Bit 17 (VM)
    /// * Bit 19 (VIP)
    /// * Bit 20
    ///
    /// Segment registers cannot be modified by this function
    ///
    /// Debug Registers may be accessed to allow a debugger to use hardware debugging. DR4 and DR5 are not defined. These require 8 bytes to store.
    /// The following checks are performed on any write to a debug register:
    /// * When writing to DR0-DR3, the value is checked as a pointer in the address space of the thread unless it is set to 0 (null) - `DEBUG_TARGET_NOT_MAPPED` is returned if the check fails
    /// * Writes to DR6 may modify only bits 0-3 and bit 14, other bits must be unmodified from the value read - `INVALID_OPERATION` is returned otherwise
    /// * Writes to DR7 may only modify bits 0-7 and bits 16-31 other bits must be set to 0, except for bit 10 which must be 1 - `INVALID_OPERATION` is returned otherwise
    /// * No breakpoint condition may be set in DR7 for I/O accesses (R/Wn=10b) - `INVALID_STATE` is returned otherwise
    /// * A breakpoint condition set in DR7 to Instruction Execution Only (R/Wn=00b) must also set the breakpoint length to 1 byte (LENn=00b) - `INVALID_STATE` is returned otherwise
    pub fn DebugWriteRegister(
        dh: HandlePtr<DebugHandle>,
        regno: u32,
        buf: *const c_void,
        meta: u32,
    ) -> SysResult;

    /// Captures Exceptions raised on the target thread.
    ///
    /// ## Errors
    ///
    /// If `dh` is not a valid `DebugHandle`, `INVALID_HANDLE` is returned.
    ///
    /// If `info_buf` does not point to suitable storage writable by the current process, `INVALID_MEMORY` is returned.
    ///
    /// ## Buffer Validity
    ///
    /// If `info_buf` does not belong to a valid mapping when the syscall is made, `INVALID_MEMORY` is returned immediately.
    /// However, if it belongs to mapped memory at the time of the syscall, and is unmapped prior to being written by the kernel, this is not checked.
    /// This results in `SIGSEGV` being delivered to the thread.
    ///
    /// ## Breakpoints
    ///
    /// Breakpoint Traps are reported to [`DebugCaptureExceptions`] as Exception type `df1ddb62-49c5-560f-86ab-1910471570b1` (DebugTrap).
    /// This exception is not reported to
    pub fn DebugCaptureExceptions(
        dh: HandlePtr<DebugHandle>,
        info_buf: *mut ExceptionInfo,
    ) -> SysResult;

    /// Blocks the current thread until a capture is made by a prior call to `DebugCaptureSignal`.
    ///
    /// A succesful return from this function *synchronizes-with* the delivery of the signal that triggered the capture, and the modification of `info_buf`
    ///
    /// ## Errors
    ///
    /// If `dh` is not a valid `DebugHandle`, `INVALID_HANDLE` is returned.
    ///
    /// If `dh` has not been configured to capture any signals, `INVALID_STATE` is returned.
    ///
    /// If the current thread is interrupted, `INTERRUPTED` is returned.
    ///
    /// If the blocking timeout expires, `TIMEOUT` is returned.
    ///
    pub fn DebugAwaitCapture(dh: HandlePtr<DebugHandle>) -> SysResult;
    /// Polls if a capture is made by a prior call to `DebugCaptureSignal`.
    ///
    /// A succesful return from this function *synchronizes-with* the delivery of the signal that triggered the capture, and the modification of `info_buf`
    /// ## Errors
    ///
    /// If `dh` is not a valid `DebugHandle`, `INVALID_HANDLE` is returned.
    ///
    /// If `dh` has not been configured to capture any signals, `INVALID_STATE` is returned.
    ///
    /// If a capture has not been made since the last call to `DebugCaptureSignal` or `DebugResume`, `PENDING` is returned.
    pub fn DebugPollCapture(dh: HandlePtr<DebugHandle>) -> SysResult;

    /// Resumes all thread suspended by this handle.
    /// ## Errors
    ///
    /// If `dh` is not a valid `DebugHandle`, `INVALID_HANDLE` is returned.
    ///
    /// If `dh` is not suspended, `INVALID_STATE` is returned.
    pub fn DebugResume(dh: HandlePtr<DebugHandle>) -> SysResult;
}
