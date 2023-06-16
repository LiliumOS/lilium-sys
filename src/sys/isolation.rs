use core::ffi::c_ulong;

use crate::uuid::Uuid;

use super::{
    device::DeviceHandle,
    fs::FileHandle,
    handle::{Handle, HandlePtr},
    kstr::KStrCPtr,
    result::SysResult,
};

#[repr(transparent)]
pub struct NamespaceHandle(Handle);

#[repr(C)]
pub struct IsolationDeviceDescriptor {
    pub devid: Uuid,
    pub redirect_dev: HandlePtr<DeviceHandle>,
}

pub const DEVICE_GROUP_ALL: u32 = !0;
pub const DEVICE_GROUP_STANDARD: u32 = 0xFFFF;
pub const DEVICE_GROUP_STORAGE_DEVICES: u32 = 0x4;
pub const DEVICE_GROUP_VIRTUAL_FILESYSTEM: u32 = 0x8;
pub const DEVICE_GROUP_ALL_FILESYSTEMS: u32 = 0xF;
pub const DEVICE_GROUP_CLOCKS: u32 = 0x10;
pub const DEVICE_GROUP_RAND_DEVICES: u32 = 0x20;

pub const ISOLATE_PROCESSES_EXPOSE_SELF: u32 = 0x1;

#[allow(improper_ctypes)]
extern "C" {
    pub fn CreateNamespace(handle: *mut HandlePtr<NamespaceHandle>) -> SysResult;
    pub fn DisposeNamespace(handle: HandlePtr<NamespaceHandle>) -> SysResult;

    pub fn IsolateDevices(
        ns: HandlePtr<NamespaceHandle>,
        devgroup: u32,
        expose_devices: *const IsolationDeviceDescriptor,
        expose_devices_len: c_ulong,
    ) -> SysResult;

    pub fn IsolateMounts(
        handle: HandlePtr<NamespaceHandle>,
        allowed_mounts_array: *const KStrCPtr,
        allowed_mounts_len: usize,
    ) -> SysResult;

    pub fn IsolateFileSystem(
        handle: HandlePtr<NamespaceHandle>,
        base: HandlePtr<FileHandle>,
    ) -> SysResult;

    pub fn IsolateProcesses(ns: HandlePtr<NamespaceHandle>, flags: u32) -> SysResult;

    pub fn InstallNamespace(handle: HandlePtr<NamespaceHandle>) -> SysResult;

}
