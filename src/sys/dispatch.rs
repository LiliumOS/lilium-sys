use core::{ffi::c_ulong, mem::MaybeUninit};

use super::{kstr::KSlice, result::SysResult};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct SystemFunction {
    /// The syscall number to execute. On return this contains the [`SysResult`] return value
    pub number_and_status: usize,
    /// The parameters of the operation.
    /// This refers to discrete parameters, even if they are smaller than a `usize` (for example, a `u32` takes up an entire parameter slot even on a 64-bit target).
    /// Note that some parameters may be two `usize`'s long and thus take up two parameter slots (without being passed indirectly).
    /// Parameters longer than 2 `usizes` will be passed indirectly always, even if the API uses a direct parameter (for example a [`Uuid`][crate::uuid::Uuid] on 32-bit).
    pub params: [MaybeUninit<usize>; 6],
}

#[cfg(feature = "base")]
unsafe extern "system" {
    /// Performs multiple system calls in a single operation.
    /// Each element of `ops` is a discrete system call, which is executed in a batch. The outcomes are placed in the [`SystemFunction::number_and_status`] field before return
    ///
    /// ## Invalid System Calls
    /// Certain system calls cannot be invoked through this function:
    /// * Blocking system calls - any blocking system call run through this function results in a `KERNEL_FUNCTION_WILL_BLOCK` error (note that this is the case even if the actually executed operation would not block):
    ///   * Event based functions should use [`BlockOnEventsAny`][crate::sys::event::BlockOnEventsAny] or [`BlockOnEventsAll`][crate::sys::event::BlockOnEventsAll],
    ///   * IO routines must configured in [`MODE_NONBLOCKING`][crate::sys::io::MODE_NONBLOCKING] or [`MODE_ASYNC`][crate::sys::io::MODE_ASYNC] (even if the requested I/O operation is guaranteed not to block) to be dispatchable by this function
    /// * [`DispatchSystemFunctions`] itself cannot be invoked through this function, doing so returns `UNSUPPORTED_KERNEL_FUNCTION` for that system call.
    pub unsafe fn DispatchSystemFunctions(ops: KSlice<SystemFunction>) -> SysResult;
}
