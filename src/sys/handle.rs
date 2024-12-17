//!
//! Handles are objects that are abstract kernel resource which is owned by a single thread.
//! Handles are active capabilities, their existance represents a permission (or multiple permissions) to an object, and threads using them do not need specific permissions to use the handle.
//!
//! Handles are represented by a pointer type, which points to the resource. The [`HandlePtr<H>`] type represents this pointer. Handle pointers are not dereferenceable in userspace,
//!      it is undefined behaviour in userspace to read or write from them (in most cases, this will generate a Access Violation Exception).
//!
//! Handles are single-thread objects and cannot be shared (except actively, via system calls like [`ShareHandle`]) with other threads or processes, even if placed into shared memory.
//! Using a handle created on a different thread is userspace undefined behaviour (in most cases, this will yield an `INVALID_HANDLE` error).
//!
//! Handles are strongly typed, both statically and dynamically. Handles are returned to userspace as specific instantiations of [`HandlePtr<H>`] in accordance with the type of the handle,
//!  and the handle.
//! Using a handle of the wrong type to perform an action (except when the type the handle is used as is a supertype of the specific handle type), an `INVALID_HANDLE` error is returned by the system call.
//!
//! Handles may have attached permissions or capabilities, which determine the actions a handle may perform, known as rights. These permissions belong to the handle itself.
//! Not all handles have associated permissions, and only a limited permissions related to the object the handle refers to, such as a thread or process.
//! Each handle type describes what rights each handle has.
//! In the case of handles with standard (kernel, process, or thread) permissions, an ambient permission check occurs at the handle's creation for each handle right using the current thread's active security context.
//! Each permission check that succeeds attaches that permission to the handle. There will typically be methods to drop existing permissions and attach new ones to a handle.
//!
//! Note that if permissions aren't present on a handle, the associated action cannot be performed, *even* if the calling thread has that permission in its security context.
//! To add or restore permissions not present on a handle, use the [`GainHandleRight`] syscall. This syscall will perform the appropriate permission check before granting the right.
//!

use core::ffi::c_ulong;

use super::{kstr::KStrCPtr, result::SysResult};

/// An opaque type that represents any object referred to by a handle
pub struct Handle(());

/// A pointer that represents an opaque handle to an object.
/// This type has the same layout as a pointer, but may not be dereferencead as a pointer.
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
    /// Creates a null handle pointer of the type.
    /// Null handles are never valid for operations on a handle, any system call given this handle will return `INVALID_HANDLE`, unless the behaviour is otherwise described.
    /// This may be used as a reliable init state, or a sentinel value for system calls that may be used without a proper handle to refer to an ambeit handle.
    ///
    /// The result is bitwise-equalivant to a null pointer
    pub const fn null() -> Self {
        Self(core::ptr::null_mut())
    }

    /// Statically converts between handles of different types. This does not alter the value of the handle, or it's dynamic type,
    /// and is typically incorrect except to cast to the generic [`Handle`] type,
    ///  or to upcast (or downcast) to a supertype (or correct subtype).
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

/// A Handle that is 16 bytes in size, regardless of the size of [`HandlePtr<T>`].
/// The excess (padding) bytes must be initialized to `0`.
#[repr(C, align(16))]
pub struct WideHandle<T> {
    /// The handle stored by the [`WideHandle<T>`]. This is guaranteed to be at offset 0 of the struct.
    pub handle: HandlePtr<T>,
    #[doc(hidden)]
    pub __pad: [u32; (16 - core::mem::size_of::<HandlePtr<Handle>>()) >> 2],
}

impl<T> WideHandle<T> {
    /// The Constant `null`. This can be used to initialize the padding bytes with struct member update syntax
    pub const NULL: Self = Self {
        handle: HandlePtr::null(),
        __pad: [0; (16 - core::mem::size_of::<HandlePtr<Handle>>()) >> 2],
    };
}

impl<T> Clone for WideHandle<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for WideHandle<T> {}

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
pub const HANDLE_TYPE_NAMESPACE: c_ulong = 6;
pub const HANDLE_TYPE_ENVMAP: c_ulong = 7;

/// Causes upgrade requests to skip privilege checks
pub const SHARE_FLAG_UPGRADE_PRIVILEGED: u32 = 1;

#[allow(improper_ctypes)]
unsafe extern "C" {
    pub fn ShareHandle(
        shared_handle: *mut SharedHandlePtr,
        hdl: HandlePtr<Handle>,
        flags: u32,
    ) -> SysResult;
    pub fn UnshareHandle(hdl: HandlePtr<Handle>) -> SysResult;
    pub fn UpgradeSharedHandle(
        hdlout: *mut HandlePtr<Handle>,
        shared_handle: SharedHandlePtr,
    ) -> SysResult;
    pub fn IdentHandle(hdl: HandlePtr<Handle>) -> SysResult;

    /// Validates that the handle right `right` is is present on `hdl`.
    ///
    /// Returns `OK` if the right is present, and `PERMISSION` if the right is not.
    /// Note that rights that aren't defined for a handle type are valid and are treated as if they aren't present.
    ///
    /// ## Errors
    ///
    /// Returns `INVALID_HANDLE` if `hdl` is not a valid handle.
    ///
    /// Return `INVALID_MEMORY` if `right` does not point to valid memory.
    /// Returns `INVALID_STRING` if `right` is not a valid UTF-8 string.
    ///
    /// Returns `PERMISSION` if  `hdl` does not possess the named access `right`
    pub fn CheckHandleRight(hdl: HandlePtr<Handle>, right: KStrCPtr) -> SysResult;
    /// Drops the specified handle right `right` from `hdl`. `hdl` can no longer perform operations associated with `right`.
    ///
    /// An error only occurs if the operation cannot be performed (such as being given an invalid handle, or an invalid string),
    ///  the call will always return `OK` regardless of whether the specified `right` is present on the handle (or even is a defined permission type)
    ///
    /// ## Errors
    /// Returns `INVALID_HANDLE` if `hdl` is not a valid handle.
    ///
    /// Return `INVALID_MEMORY` if `right` does not point to valid memory.
    /// Returns `INVALID_STRING` if `right` is not a valid UTF-8 string.
    pub fn DropHandleRight(hdl: HandlePtr<Handle>, right: KStrCPtr) -> SysResult;

    /// Drops all rights from the specified handle
    ///
    /// ## Errors
    /// Returns `INVALID_HANDLE` if `hdl` is not a valid handle.
    pub fn DropAllHandleRights(hdl: HandlePtr<Handle>) -> SysResult;

    /// Grants the specified named right to `hdl` if the current thread has the required permissions.
    ///
    /// The permission check is performed regardless of whether `hdl` already has the given right
    ///
    /// ## Errors
    /// Returns `INVALID_HANDLE` if `hdl` is not a valid handle.
    ///
    /// Returns `INVALID_MEMORY` if `right` does not point to valid memory.
    /// Returns `INVALID_STRING` if `right` is not a valid UTF-8 string.
    ///
    /// Returns `PERMISSION` if the thread does not have the required permission.
    /// Returns `INVALID_OPERATION` if the specified `right` cannot be applied to the given handle.
    pub fn GrantHandleRight(hdl: HandlePtr<Handle>, right: KStrCPtr) -> SysResult;
}
