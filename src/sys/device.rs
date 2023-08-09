use core::ffi::{c_long, c_ulong, c_void, VaList};

use crate::{security::SecurityContext, uuid::Uuid};

use self::udev::DeviceCommandParameter;

use super::{
    fs::FileHandle,
    handle::{Handle, HandlePtr},
    io::IOHandle,
    kstr::{KStrCPtr, KStrPtr},
    result::SysResult,
};

pub mod udev;

#[repr(C)]
pub struct BlockDeviceConfiguration {
    pub label: KStrCPtr,
    pub acl: HandlePtr<FileHandle>,
    pub optimistic_io_size: u64,
    pub base: c_ulong,
    pub extent: c_long,
}

#[repr(C)]
pub struct CharDeviceConfiguration {
    pub label: KStrCPtr,
    pub acl: HandlePtr<FileHandle>,
    pub optimistic_io_size: u64,
}

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

#[repr(C)]
pub struct VirtualFSDescriptor {
    pub label: KStrCPtr,
    pub acl: HandlePtr<FileHandle>,
    pub buffer_page: *mut c_void,
}

#[repr(C)]
pub struct RandomSourceDescriptor {
    pub label: KStrCPtr,
    pub acl: HandlePtr<FileHandle>,
    pub buffer_page: *mut c_void,
    pub gen_random_bytes_impl: unsafe extern "C" fn(*mut c_void, c_ulong) -> SysResult,
}

#[repr(C)]
pub struct ClockDescriptor {
    pub label: KStrCPtr,
    pub acl: HandlePtr<FileHandle>,
}

#[allow(improper_ctypes)]
extern "C" {
    pub fn CreateBlockDevice(
        id: *mut Uuid,
        backing_hdl: HandlePtr<IOHandle>,
        cfg: *const BlockDeviceConfiguration,
    ) -> SysResult;
    pub fn RemoveBlockDevice(backing_hdl: HandlePtr<IOHandle>) -> SysResult;
    pub fn CreateCharDevice(
        id: *mut Uuid,
        backing_hdl: HandlePtr<IOHandle>,
        cfg: *const CharDeviceConfiguration,
    ) -> SysResult;
    pub fn RemoveCharDevice(backing_hdl: HandlePtr<IOHandle>) -> SysResult;

    pub fn OpenBlockDevice(hdl: *mut HandlePtr<DeviceHandle>, id: Uuid) -> SysResult;
    pub fn OpenCharDevice(hdl: *mut HandlePtr<DeviceHandle>, id: Uuid) -> SysResult;
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

    /// Issues a Command to a device. The supported commands are device specific, and the parameters for each command is command specific
    pub fn IssueDeviceCommand(hdl: HandlePtr<DeviceHandle>, cmd: *const Uuid, ...) -> SysResult;

    pub fn MountFilesystem(
        resolution_base: HandlePtr<FileHandle>,
        path: KStrCPtr,
        devid: Uuid,
        opts: *const MountOptions,
    ) -> SysResult;

    pub fn CreateVirtualFSDevice(
        devid: *mut Uuid,
        vfsdesc: *const VirtualFSDescriptor,
    ) -> SysResult;

    pub fn CreateRandomDevice(devid: *mut Uuid, rdevdesc: RandomSourceDescriptor) -> SysResult;

    pub fn RegisterDeviceCommand(
        devid: *const Uuid,
        cmdid: *mut Uuid,
        callback: unsafe extern "C" fn(
            cmdid: *const Uuid,
            callctx: HandlePtr<SecurityContext>,
            ...
        ) -> SysResult,
        callback_stack: *mut c_void,
        sigtys: *const DeviceCommandParameter,
        param_count: c_ulong,
    ) -> SysResult;
}
