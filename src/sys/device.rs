use core::ffi::{c_ulong, c_long, c_void};

use crate::uuid::Uuid;

use super::{io::IOHandle, kstr::{KStrCPtr, KStrPtr}, handle::{HandlePtr, Handle}, result::SysResult, fs::FileHandle};


#[repr(C)]
pub struct BlockDeviceConfiguration{
    pub label: KStrCPtr,
    pub optimistic_io_size: u64,
    pub acl: HandlePtr<FileHandle>,
    pub base: c_ulong,
    pub extent: c_long,
}

#[repr(C)]
pub struct CharDeviceConfiguration{
    pub label: KStrCPtr,
    pub optimistic_io_size: u64,
    pub acl: HandlePtr<FileHandle>,
}



#[repr(transparent)]
pub struct DeviceHandle(Handle);

/// Treats every object in the mounted filesystem as having the `default_acl`
pub const MOUNT_REPLACE_ACLS: u32 = 0x01;
/// Enables the use of InstallSecurityContext and legacy SUID/SGID bits on mounted objects.
/// Requires the MountPrivilagedExec kernel permission
pub const MOUNT_ALLOW_PRIVILAGED: u32 = 0x02;


#[repr(C)]
pub struct MountOptions{
    pub default_acl: HandlePtr<FileHandle>,
    pub flags: u32,
}

#[repr(C)]
pub struct VirtualFSDescriptor{

}

#[allow(improper_ctypes)]
extern "C"{
    pub fn CreateBlockDevice(id: *mut Uuid, backing_hdl: HandlePtr<IOHandle>, cfg: *const BlockDeviceConfiguration) -> SysResult;
    pub fn RemoveBlockDevice(backing_hdl: HandlePtr<IOHandle>) -> SysResult;
    pub fn CreateCharDevice(id: *mut Uuid, backing_hdl: HandlePtr<IOHandle>, cfg: *const CharDeviceConfiguration) -> SysResult;
    pub fn RemoveCharDevice(backing_hdl: HandlePtr<IOHandle>) -> SysResult;

    pub fn OpenBlockDevice(hdl: *mut HandlePtr<DeviceHandle>, id: Uuid) -> SysResult;
    pub fn OpenCharDevice(hdl: *mut HandlePtr<DeviceHandle>, id: Uuid) -> SysResult;
    pub fn CloseDevice(hdl: HandlePtr<DeviceHandle>) -> SysResult;


    pub fn GetDeviceLabel(hdl: HandlePtr<DeviceHandle>, label: *mut KStrPtr) -> SysResult;
    pub fn GetOptimisticIOSize(hdl: HandlePtr<DeviceHandle>, io_size: *mut u64) -> SysResult;
    pub fn GetDeviceId(hdl: HandlePtr<DeviceHandle>,id: *mut Uuid) -> SysResult;
    
    pub fn GetFileDeviceLabel(hdl: HandlePtr<FileHandle>, label: *mut KStrPtr) -> SysResult;
    pub fn GetFileOptimisticIOSize(hdl: HandlePtr<FileHandle>, io_size: *mut u64) -> SysResult;
    pub fn GetFileDeviceId(hdl: HandlePtr<FileHandle>, id: *mut Uuid) -> SysResult;

    pub fn MountFilesystem(resolution_base: HandlePtr<FileHandle>, path: KStrCPtr, devid: Uuid, opts: *const MountOptions) -> SysResult;

    pub fn CreateVirtualFSDevice(devid: *mut Uuid, vfsdesc: *const VirtualFSDescriptor) -> SysResult;
}