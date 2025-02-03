//! Interfaces for managing and accessing devices in userspace
//!
//! Device operations belong to subsystem 2 (io subsystem)
//!
//! ### Handle Rights
//!
//! [`DeviceHandle`]s are a type of [`IOHandle`]. All access rights for [`IOHandle`]s apply to [`DeviceHandle`]s.
//! Additionally, seven access rights apply specifically to [`DeviceHandle`]s, which correspond to ACL permission checks on the Device's ACL:
//! * `MOUNT_FILESYSTEM`: Allows using the [`MountDevice`] system call

use core::ffi::{c_long, c_ulong, c_void};

use udev::DeviceInvocationContext;

use crate::{sys::permission::SecurityContext, uuid::Uuid};
use core::mem::MaybeUninit;

#[cfg(doc)]
use crate::sys::io::{CHAR_RANDOMACCESS, CHAR_READABLE, CHAR_SEEKABLE, CHAR_WRITABLE};

use super::{
    fs::FileHandle,
    handle::{Handle, HandlePtr},
    io::IOHandle,
    isolation::NamespaceHandle,
    kstr::{KCSlice, KSlice, KStrCPtr, KStrPtr},
    option::ExtendedOptionHead,
    result::SysResult,
};

pub mod udev;

/// Configuration for a block device created by [`CreateBlockDevice`]
#[repr(C)]
pub struct BlockDeviceConfiguration {
    /// A user-friendly name for the block device
    pub label: KStrCPtr,
    /// A [`FileHandle`] that represents an access control list, which specifies the access permisions of the created device
    pub acl: HandlePtr<FileHandle>,
    /// Specifies the number of bytes which the device reports as "Optimistic", IE. performing I/O operations of this size is at least as efficient as performing I/O operations of any smaller size
    pub optimistic_io_size: c_ulong,
    /// Specifies the base of a [`CHAR_RANDOMACCESS`] `IOHandle` to expose
    ///
    /// If the handle does not have [`CHAR_RANDOMACCESS`], this must be set to `0`
    pub base: c_ulong,
    /// Specifies the extent (maximum size) of the `IOHandle` to expose
    pub extent: c_long,
}

/// Configuraton for a charater device reated by [`CreateCharDevice`]
#[repr(C)]
pub struct CharDeviceConfiguration {
    /// A user-friendly name for the character device
    pub label: KStrCPtr,
    /// A [`FileHandle`] that represents an access control list, which specifies the access permisions of the created device
    pub acl: HandlePtr<FileHandle>,
    /// Specifies the number of bytes which the device reports as "Optimistic", IE. performing I/O operations of this size is at least as efficient as performing I/O operations of any smaller size
    pub optimistic_io_size: u64,
}

/// A Handle to a device
#[repr(transparent)]
pub struct DeviceHandle(Handle);

/// Treats every object in the mounted filesystem as having the `default_acl`
///
/// This is default if the filesystem does not support ACLs or Legacy Permisions (such as FAT32)
pub const MOUNT_REPLACE_ACLS: u32 = 0x01;
/// Enables the use of InstallSecurityContext and legacy SUID/SGID bits on mounted objects.
/// Requires the MountPrivilagedExec kernel permission
pub const MOUNT_ALLOW_PRIVILAGED: u32 = 0x02;
/// Treats every object in the mounted filesystem as having `default_acl` if the filesystem only supports legacy permissions
///
/// Note that some filesystems may support a form of ACL, but be considered to only support legacy permissions (for example, ext4's posix acl support).
/// This flag will override the ACLs on objects on which such ACLs are present
pub const MOUNT_REPLACE_LEGACY_PERMISSIONS: u32 = 0x04;

/// Specifies options for [`MountFilesystem`]
#[repr(C)]
pub struct MountOptions {
    /// The default ACL to use if the filesystem does not support permissions or where replacement is required
    pub default_acl: HandlePtr<FileHandle>,
    /// flags for the mount operation
    pub flags: u32,
    /// If the filesystem uses legacy permissions (or supports only posix acls, rather than enhanced dacls), then use this principal map given to map to Lilium principals.
    /// The IOHandle must have `CHAR_READ` and `CHAR_SEEK`. If it does not have `CHAR_RANDOMACCESS` then the behaviour is undefined if the thread calls `IOSeek`, or performs an I/O operation on the handle.
    pub legacy_principal_map: HandlePtr<IOHandle>,
}

/// Checks whether the feature supports read operations (such as obtaining the offset of a clock device, or polling a random device)
///
/// ## Notes
///
/// When checking the `BasicIo` feature, this does not take handle capabilities into account.
pub const DEVICE_FEATURE_OPTION_READ: u32 = 0x01;
/// Checks whether the feature supports write operations (such as resetting a clock device, or reseeding a random device)
pub const DEVICE_FEATURE_OPTION_WRITE: u32 = 0x02;
/// Skip performing access control checks for this feature
pub const DEVICE_FEATURE_OPTION_IGNORE_AC: u32 = 0x8000;
#[repr(C)]
pub struct DeviceFeature {
    pub feature_name: KStrCPtr,
    pub feature_options: u32,
}

/// The Number of bytes for the body of a [`SysInfoRequest`] - large enough to store the larger of 8 pointers and 64 bytes.
pub const DEVICE_COMMAND_BODY_SIZE: usize = if core::mem::size_of::<usize>() > 8 {
    core::mem::size_of::<usize>() * 8
} else {
    64
};

#[repr(C, align(32))]
#[derive(Copy, Clone)]
pub struct DeviceFunctionUnknown {
    pub head: ExtendedOptionHead,
    pub content: [MaybeUninit<u8>; DEVICE_COMMAND_BODY_SIZE],
}

#[repr(C, align(32))]
#[derive(Copy, Clone)]
pub union DeviceFunction {
    pub head: ExtendedOptionHead,
    unknown: DeviceFunctionUnknown,
}

pub const DEVICE_FUNCTION_MARKER_TY_SLICE_BUFFER: u32 = 0x01;
pub const DEVICE_FUNCTION_MARKER_TY_HANDLE: u32 = 0x02;

#[repr(C, align(32))]
pub struct DeviceFunctionMarkers {
    pub offset: usize,
    pub ty: u32,
    pub elem_size: usize,
}

#[expect(improper_ctypes)]
unsafe extern "system" {

    /// Creates a new block device backed by `backing_hdl`, with the specified configuration.
    ///
    /// `backing_hdl` must be seekable [`CHAR_SEEKABLE`]. If it is random-access [`CHAR_RANDOMACCESS`] that will be exposed to handles opened to it.
    ///
    /// The readability and writability of the device are those of the `IOHandle`,
    ///  but are limited by the permissions of the process that opens the device and the `acl` specified in `cfg`.
    ///
    /// `id` can be assigned by either the process or the kernel. If `id` is set to the nil UUID (all zeroes), the kernel will generate a device id and store it in `id`.
    ///  Otherwise, the device is assigned the `id` specified in `id` if it is unused.
    ///
    /// If `ns` is specified, then the device is created inside that namespace only. Otherwise, the device is created in the device scope of the current thread.
    ///
    /// The returned device handle has full rights to the device (absent any basic [`IOHandle`] rights not present on `backing_hdl`), regardless of the ACL.
    ///
    /// ## Errors
    ///
    /// If `backing_hdl` is not a valid `IOHandle`, returns `INVALID_HANDLE`.
    ///
    /// If `backing_hdl` was previouslly passed to either this function or [`CreateCharDevice`] and the created device has not been remoed, returns `DEVICE_UNAVAILABLE`.
    ///
    /// If `backing_hdl` does not have [`CHAR_SEEKABLE`], returns `INVALID_OPERATION`. If `backing_hdl` does not have [`CHAR_RANDOMACCESS`] and `cfg.base` is not set to `0`, returns `INVALID_OPERATION`.
    ///
    /// If the current thread does not have the kernel permission `CREATE_BLOCK_DEVICE`, returns `PERMISSION`.
    ///
    /// If `id` is set to an explicit ID, `ns` is not an explicit namespace handle that has devices isolated, and the current thread does not have `ASSIGN_DEVICE_ID` permission,
    ///  returns `PERMISSION`.
    ///
    /// If `id` is set to an explicit ID, and that id is already in use by a device within the device scope of the specified namespace, returns `ALREADY_EXISTS`.
    ///
    pub fn CreateBlockDevice(
        dev: *mut HandlePtr<DeviceHandle>,
        id: *mut Uuid,
        backing_hdl: HandlePtr<IOHandle>,
        cfg: *const BlockDeviceConfiguration,
        ns: HandlePtr<NamespaceHandle>,
    ) -> SysResult;
    /// Removes the block device specified by `hdl`.
    ///
    /// `hdl` must have the "DeregisterDevice"
    ///
    pub fn DeregisterDevice(hdl: HandlePtr<DeviceHandle>) -> SysResult;
    /// Creates a new character device, backed by a given `IOHandle`.
    ///
    /// Character devices are not seekable or random access - `backing_hdl` may be non-seekable (Does not have `CHAR_SEEKABLE`), and handles referring to it will not have the characteristics `CHAR_SEEKABLE` or `CHAR_RANDOM_ACCESS`, regardless of the underlying handle
    ///
    ///
    /// The readability and writability of the device are those of the `IOHandle`,
    ///  but are limited by the permissions of the process that opens the device and the `acl` specified in `cfg`.
    ///
    /// `id` can be assigned by either the process or the kernel. If `id` is set to the nil UUID (all zeroes), the kernel will generate a device id and store it in `id`.
    ///  Otherwise, the device is assigned the `id` specified in `id` if it is unused.
    ///
    /// If `ns` is specified, then the device is created inside that namespace only. Otherwise, the device is created in the device scope of the current thread.
    /// ## Errors
    ///
    /// If `backing_hdl` is not a valid `IOHandle`, returns `INVALID_HANDLE`.
    ///
    /// If `backing_hdl` was previouslly passed to either this function or [`CreateBlockDevice`] and the created device has not been remoed, returns `DEVICE_UNAVAILABLE`.
    ///
    /// If `backing_hdl` does not have [`CHAR_SEEKABLE`], returns `INVALID_OPERATION`. If `backing_hdl` does not have [`CHAR_RANDOMACCESS`] and `cfg.base` is not set to `0`, returns `INVALID_OPERATION`.
    ///
    /// If the current thread does not have the kernel permission `CREATE_BLOCK_DEVICE`, returns `PERMISSION`.
    ///
    /// If `id` is set to an explicit ID, `ns` is not an explicit namespace handle that has devices isolated, and the current thread does not have `ASSIGN_DEVICE_ID` permission,
    ///  returns `PERMISSION`.
    ///
    /// If `id` is set to an explicit ID, and that id is already in use by a device within the device scope of the specified namespace, returns `ALREADY_EXISTS`.
    ///
    pub fn CreateCharDevice(
        dev: *mut HandlePtr<DeviceHandle>,
        id: *mut Uuid,
        backing_hdl: HandlePtr<IOHandle>,
        cfg: *const CharDeviceConfiguration,
        ns: HandlePtr<NamespaceHandle>,
    ) -> SysResult;

    /// Opens a device by it's id, if the given device exists.
    ///
    /// ## Errors
    ///
    /// If `id` does not identify a valid device, returns `UNKNOWN_DEVICE`
    ///
    ///
    pub fn OpenDevice(hdl: *mut HandlePtr<DeviceHandle>, id: Uuid) -> SysResult;
    pub fn CloseDevice(hdl: HandlePtr<DeviceHandle>) -> SysResult;

    pub fn GetDeviceLabel(hdl: HandlePtr<DeviceHandle>, label: *mut KStrPtr) -> SysResult;
    pub fn GetOptimisticIOSize(hdl: HandlePtr<DeviceHandle>, io_size: *mut u64) -> SysResult;
    pub fn GetDeviceId(hdl: HandlePtr<DeviceHandle>, id: *mut Uuid) -> SysResult;

    pub fn GetFileDeviceLabel(hdl: HandlePtr<FileHandle>, label: *mut KStrPtr) -> SysResult;
    pub fn GetFileOptimisticIOSize(hdl: HandlePtr<FileHandle>, io_size: *mut u64) -> SysResult;
    pub fn GetFileDeviceId(hdl: HandlePtr<FileHandle>, id: *mut Uuid) -> SysResult;
    pub fn OpenDeviceFromFile(
        devhdl: *mut HandlePtr<DeviceHandle>,
        file: HandlePtr<FileHandle>,
    ) -> SysResult;

    pub fn MountFilesystem(
        resolution_base: HandlePtr<FileHandle>,
        path: KStrCPtr,
        devid: Uuid,
        opts: *const MountOptions,
    ) -> SysResult;

    /// Tests whether `hdl` supports the specified features,
    ///
    /// ## Errprs
    ///
    /// Returns `INVALID_HANDLE` if `hdl` is not a valid device Handle
    ///
    /// Returns `INVALID_MEMORY` if any pointer in `features` is null or invalid.
    ///
    /// Returns `UNSUPPORTED_OPERATION` if any of the named features in `features` is not supported by the device designated by `hdl`.
    ///
    /// Returns `INVALID_OPERATION` if any of the named features in `features` are supported, but not in the requested mode(s).
    ///
    /// Returns `PERMISSION` if any of the named features in `features` are Access Controlled and the Access Control checks are not marked as ignorable for that feature, the required right is not present in the handle, and permission is denied to the calling thread to obtain the required right.
    pub unsafe fn TestDeviceFeature(
        hdl: HandlePtr<DeviceHandle>,
        features: *const KCSlice<DeviceFeature>,
    ) -> SysResult;

    pub unsafe fn InvokeDeviceFunctions(
        hdl: HandlePtr<DeviceHandle>,
        functions: KSlice<DeviceFunction>,
    ) -> SysResult;

    /// Registers the current thread as a support provider for device function `fn_id`.
    /// When the function is invoked, the exception `DeviceFunction` is thrown with `data` pointing to `ctx_buffer` which is populated before the handler is invoked.
    ///
    /// `ctx_buffer` is only valid within the exception handler,
    ///  but is guaranteed not to be overwritten during (each invocation is separated such that resuming from one handler will synchronize with the next incoming exception).
    ///
    /// On any given device
    pub unsafe fn RegisterDeviceFunction(
        hdl: HandlePtr<DeviceHandle>,
        fn_id: Uuid,
        ctx_buffer: *mut DeviceInvocationContext,
    ) -> SysResult;
}
