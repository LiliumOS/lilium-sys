use core::ffi::c_ulong;

use super::result::SysResult;

pub struct Handle(u8);

#[repr(transparent)]
pub struct HandlePtr<T>(*mut T);

impl<T> core::fmt::Pointer for HandlePtr<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> core::fmt::Debug for HandlePtr<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> Clone for HandlePtr<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for HandlePtr<T> {}

impl<T> core::hash::Hash for HandlePtr<T> {
    fn hash<H: core::hash::Hasher>(&self, hasher: &mut H) {
        self.0.hash(hasher);
    }
}

impl<T> core::cmp::PartialEq for HandlePtr<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl<T> core::cmp::Eq for HandlePtr<T> {}

impl<T> HandlePtr<T> {
    pub const fn null() -> Self {
        Self(core::ptr::null_mut())
    }
    pub const fn cast<U>(self) -> HandlePtr<U> {
        HandlePtr(self.0.cast())
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, Hash, PartialEq, Eq, Debug)]
pub struct SharedHandlePtr(*mut Handle);

impl core::fmt::Pointer for SharedHandlePtr {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        self.0.fmt(f)
    }
}

unsafe impl Send for SharedHandlePtr {}
unsafe impl Sync for SharedHandlePtr {}

pub const HANDLE_TYPE_PROC: c_ulong = 1;
pub const HANDLE_TYPE_THREAD: c_ulong = 2;
pub const HANDLE_TYPE_IO: c_ulong = 3;
pub const HANDLE_SUBTYPE_IO_FILE: c_ulong = 0x10000003;
pub const HANDLE_SUBTYPE_IO_DEV: c_ulong = 0x20000003;
pub const HANDLE_SUBTYPE_IO_PIPE_READ: c_ulong = 0x30000003;
pub const HANDLE_SUBTYPE_IO_PIPE_WRITE: c_ulong = 0x40000003;
pub const HANDLE_SUBTYPE_IO_SOCKET: c_ulong = 0x50000003;
pub const HANDLE_SUBTYPE_IO_SERVER: c_ulong = 0x60000003;
pub const HANDLE_SUBTYPE_IO_MEMBUF: c_ulong = 0x70000003;
pub const HANDLE_SUBTYPE_IO_IPCCON: c_ulong = 0x80000003;
pub const HANDLE_SUBTYPE_IO_IPCSERVER: c_ulong = 0x90000003;
pub const HANDLE_TYPE_DEBUG: c_ulong = 4;
pub const HANDLE_TYPE_SECURITY: c_ulong = 5;

#[allow(improper_ctypes)]
extern "C" {
    pub fn ShareHandle(shared_handle: *mut SharedHandlePtr, hdl: HandlePtr<Handle>) -> SysResult;
    pub fn UnshareHandle(hdl: HandlePtr<Handle>) -> SysResult;
    pub fn UpgradeSharedHandle(
        hdlout: HandlePtr<Handle>,
        shared_handle: SharedHandlePtr,
    ) -> SysResult;
    pub fn IdentHandle(hdl: HandlePtr<Handle>) -> SysResult;
}
