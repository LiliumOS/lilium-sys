//! Interfaces for specifying device commands in userspace

use core::ffi::{c_ulong, c_void};

use crate::sys::{
    handle::{Handle, HandlePtr},
    result::SysResult,
    thread::ThreadHandle,
};

use super::{DeviceFunction, DeviceHandle};

#[repr(transparent)]
pub struct InvokeContextHandle(Handle);

#[repr(C)]
#[non_exhaustive]
pub struct DeviceInvocationContext {
    pub dev_hdl: HandlePtr<DeviceHandle>,
    pub func: DeviceFunction,
    pub ctx_handle: HandlePtr<InvokeContextHandle>,
}

#[expect(improper_ctypes)]
unsafe extern "system" {
    pub unsafe fn GetHandle(
        r: *mut HandlePtr<Handle>,
        off: usize,
        ctx: HandlePtr<InvokeContextHandle>,
    ) -> SysResult;
    pub unsafe fn GetKBuffer(
        b: *mut c_void,
        len: *mut usize,
        elem_size: usize,
        off: usize,
        ctx: HandlePtr<InvokeContextHandle>,
    ) -> SysResult;
}
