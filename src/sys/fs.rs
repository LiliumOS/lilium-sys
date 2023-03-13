use core::ffi::{c_void, c_long};

use crate::uuid::Uuid;

use super::{handle::{Handle,HandlePtr}, kstr::{KStrCPtr, KStrPtr}, result::SysResult, ipc::IPCServerHandle, io::IOHandle, socket::SocketHandle};

#[repr(transparent)]
pub struct FileHandle(Handle);



/// Opens the file for read access
/// 
/// Requires `Read` access to the stream being opened.
pub const ACCESS_READ: u32 = 0x01;
/// Opens the file for write access
/// 
/// Requires `Write` access to the stream being opened.
pub const ACCESS_WRITE: u32 = 0x02;
/// Creates the object and stream if necessary
/// 
/// Requires `CreateObject` permission to the containing directory of the object in path resolution
pub const ACCESS_CREATE: u32 = 0x04;
/// If set together with `ACCESS_CREATE`, errors if the object exists.
/// If `ACCESS_CREATE` is not set, errors.
pub const ACCESS_CREATE_EXCLUSIVE: u32 = 0x08;
/// Locks the stream by preventing access to any operation using either `ACCESS_LOCK_SOFT` or `ACCESS_LOCK_HARD` by blocking such open attempts (unless both the live lock and open operation are using ACCESS_LOCK_SHARED)
pub const ACCESS_LOCK_SOFT: u32 = 0x10;
/// Locks the stream by preventing any other access, and blocking all open operations, unless they have `ACCESS_OVERRIDE_LOCK` set. 
/// The open operation emplyoing `ACCESS_LOCK_HARD` blocks if any other version of the file is open, unless the operation and the open file used ACCESS_LOCK_SHARED.
/// 
/// Requires `StrictLock` permission to the stream
pub const ACCESS_LOCK_HARD: u32 = 0x20;
/// Locks the file non-exclusively to other locking operations using `ACCESS_LOCK_SHARED`, but still enforces `ACCESS_LOCK_SOFT`.
pub const ACCESS_LOCK_SHARED: u32 = 0x40;
/// Does not consider the kernel permission `BYPASS_FILESYSTEM_ACCESS_CONTROL` in determining whether the file can be opened in the particular access mode
pub const ACCESS_NO_BYPASS_ACL: u32 = 0x40;
/// If `ACCESS_LOCK_HARD` or `ACCESS_LOCK_SOFT` are not used, ignores the existance of any other hard lock on the file.
/// 
/// Requires either `OverrideStrictLock` permision to the object, or `BYPASS_LOCK_EXCLUSIVE` permission to the kernel.
pub const ACCESS_OVERRIDE_LOCK: u32 = 0x80;
/// When combined with `ACCESS_CREATE`, errors if the object does not exist, but stil creates the stream.
pub const ACCESS_CREATE_STREAM_ONLY: u32 = 0x100;
/// When opening a `SymbolicLinkContent` stream, or some other streams that represent a symbolic link, open the stream content directly.
/// Incompatible with `ACCESS_WRITE`.
pub const ACCESS_LINK_STREAM_DIRECT: u32 = 0x200;

/// Performs the default operation on the stream being opened
pub const OP_STREAM_DEFAULT: u32 = 0x00;
/// Access the raw data of the stream, regardless of type.
/// Certain streams do not allow direct write access.
/// Additionally, reading from some types of metadata streams may produce unspecified results when the stream being opened belongs to a synthetic object, or is on a filesystem other than PhantomFS.
pub const OP_DATA_ACCESS: u32 = 0x01;
/// Views the stream, if possible, as a directory, and allows use in path resolution.
/// If the stream type cannot be viewed as a directory, errors.
/// 
/// If the default stream is used, and it is not a stream that is valid for directory access, the `DirectoryContent` stream of the object is used instead. 
/// This does not trigger creation of the `DirectoryContent` stream even if `ACCESS_CREATE` or `ACCESS_CREATE_EXCLUSIVE` is used - the stream must be named explicitly to recieve this behaviour
pub const OP_DIRECTORY_ACCESS: u32 = 0x02;

/// Opens the file, or the stream's access control list.
/// 
/// If an explicit stream is given, it views rows in the access control list that apply only to that stream, unless the stream is of type `SecurityDescriptor` or `LegacySecurityDescriptor`.
/// In the special case of the `Streams` stream, it views rows that apply at the object level, but not stream-level rows.
/// If the stream is `SecurityDescriptor`, it views all ACL rows. If the stream is `LegacySecurityDescriptor`, it does not access any ACL rows, but allows setting the object-level legacy unix permissions.
/// 
/// When no explicit stream is given, it allows access to the full ACL and to the legacy security descriptor.
pub const OP_ACL_ACCESS: u32 = 0x03;

/// Ignores the `access_mode` and does not open the stream.
/// This may be used to do operations on the object itself, or on stream metadata
pub const OP_NO_ACCESS: u32 = 0x04;


pub use super::io::{MODE_BLOCKING, MODE_NONBLOCKING, MODE_ASYNC};

#[repr(C)]
pub struct FileOpenOptions{
    /// If non-null, set to a `FileHandle` opened in `OP_DIRECTORY_ACCESS` mode to use in path resolution for relative paths instead of the current directory.
    pub resolution_base: HandlePtr<FileHandle>,
    /// The file path to open
    pub path: KStrCPtr,
    /// If set to a non-empty string, designates the explicit stream of the object to open.
    /// 
    /// Only certain filesystems support multiple streams (such as NTFS or PhantomFS). Other filesystems may only support a limited set of standard streams, 
    /// and the streams supported may depend on the object type and the filesystem.
    /// 
    /// Certain stream types permit multiple streams of the type. This is designated by following the stream name with a `$` then the integer number of the stream of that type.
    /// If this is not present, then the first stream of the type is used. 
    pub stream_override: KStrCPtr,
    /// The file access mode to open the stream in. This determines what operations can be performed, and the access control permissions required,
    pub access_mode: u32,
    /// The file operation mode. This determines which operations are performed.
    pub op_mode: u32,
    /// How the file handle handles blocking operations
    pub blocking_mode: u32,
    /// For `ACCESS_CREATE`, when creating the file, overrides the default access control list for the created object.
    pub create_acl: HandlePtr<FileHandle>,
}

#[repr(C)]
pub struct DirectoryInfo{
    pub fname: KStrPtr,
    pub flags: u64,
    pub acl_handle: HandlePtr<FileHandle>,
}

#[repr(C)]
pub struct ReadDaclRow{
    pub stream_name: KStrPtr,
    pub perm_name: KStrPtr,
    pub principal: Uuid,
    pub mode: u32,
}

#[repr(C)]
pub struct DaclRow{
    pub stream_name: KStrCPtr,
    pub perm_name: KStrCPtr,
    pub principal: Uuid,
    pub mode: u32,
}

#[allow(improper_ctypes)]
extern "C"{
    pub fn OpenFile(hdl: *mut HandlePtr<FileHandle>, opts: *const FileOpenOptions) -> SysResult;
    pub fn CloseFile(hdl: HandlePtr<FileHandle>) -> SysResult;
    pub fn DirectoryNext(hdl: HandlePtr<FileHandle>,state: *mut *mut c_void) -> SysResult;
    pub fn DirectoryRead(hdl: HandlePtr<FileHandle>, state: *mut c_void, info: *mut DirectoryInfo) -> SysResult;

    pub fn StreamSize(hdl: HandlePtr<FileHandle>) -> SysResult;
    pub fn ObjectSize(hdl: HandlePtr<FileHandle>, size_out: *mut u128) -> SysResult;

    pub fn CreateAcl(hdl: *mut HandlePtr<FileHandle>) -> SysResult;
    pub fn DefaultAcl(hdl: *mut HandlePtr<FileHandle>) -> SysResult;
    pub fn ObjectOwner(hdl: HandlePtr<FileHandle>, uuid: *mut Uuid) -> SysResult;
    pub fn SetObjetOwner(hdl: HandlePtr<FileHandle>, uuid: *const Uuid) -> SysResult;
    pub fn AclNextRow(hdl: HandlePtr<FileHandle>, state: *mut *mut c_void) -> SysResult;
    pub fn AclReadRow(hdl: HandlePtr<FileHandle>, state: *mut c_void, info: *mut ReadDaclRow) -> SysResult;
    pub fn AclWriteRow(hdl: HandlePtr<FileHandle>, state: *mut c_void, new_row: *const DaclRow) -> SysResult;
    pub fn AclLegacyUid(hdl: HandlePtr<FileHandle>) -> SysResult;
    pub fn AclLegacyGid(hdl: HandlePtr<FileHandle>) -> SysResult;
    pub fn AclLegacyMode(hdl: HandlePtr<FileHandle>) -> SysResult;
    pub fn AclSetLegacyUid(hdl: HandlePtr<FileHandle>, uid: c_long) -> SysResult;
    pub fn AclSetLegacyGid(hdl: HandlePtr<FileHandle>, gid: c_long) -> SysResult;
    pub fn OverwriteAcl(file_hdl: HandlePtr<FileHandle>, acl_hdl: HandlePtr<FileHandle>) -> SysResult;
    pub fn CopyAcl(acl_out: *mut HandlePtr<FileHandle>, file_hdl: HandlePtr<FileHandle>) -> SysResult;
    pub fn SetDefaultAcl(acl: HandlePtr<FileHandle>) -> SysResult;

    pub fn CreateDirectory(dir_handle: *mut HandlePtr<FileHandle>,resolution_base: HandlePtr<FileHandle>, path: KStrCPtr, acl: HandlePtr<FileHandle>) -> SysResult;

    pub fn CreateHardLink(newfile_hdl: *mut HandlePtr<FileHandle>, new_name_base: HandlePtr<FileHandle>, new_name: KStrCPtr, old_name_base: HandlePtr<FileHandle>, old_name: KStrCPtr) -> SysResult;
    pub fn AssociateName(file: HandlePtr<FileHandle>, new_name_base: HandlePtr<FileHandle>, new_name: KStrCPtr) -> SysResult;
    pub fn CreateWeakLink(newfile_hdl: *mut HandlePtr<FileHandle>, new_name_base: HandlePtr<FileHandle>, new_name: KStrCPtr, old_name_base: HandlePtr<FileHandle>, old_name: KStrCPtr) -> SysResult;
    pub fn AssociateWeakName(file: HandlePtr<FileHandle>, new_name_base: HandlePtr<FileHandle>, new_name: KStrCPtr) -> SysResult;

    pub fn UpgradeLink(newfile_hdl: *mut HandlePtr<FileHandle>, resolution_base: HandlePtr<FileHandle>, name: KStrCPtr) -> SysResult;
    pub fn DowngradeLink(resolution_base: HandlePtr<FileHandle>, name: KStrCPtr) -> SysResult;

    pub fn RenameObject(new_name_base: HandlePtr<FileHandle>, new_name: KStrCPtr, old_name_base: HandlePtr<FileHandle>, old_name: KStrCPtr) -> SysResult;
    pub fn RemoveLink(resolution_base: HandlePtr<FileHandle>, path: KStrCPtr) -> SysResult;

    pub fn CreateNamedChannel(ipc_hdl: HandlePtr<IPCServerHandle>, name_base: HandlePtr<FileHandle>, name: KStrCPtr, acl: HandlePtr<FileHandle>) -> SysResult;
    pub fn CreateNamedServer(sock_hdl: HandlePtr<SocketHandle>, name_base: HandlePtr<FileHandle>, name: KStrCPtr, acl: HandlePtr<FileHandle>) -> SysResult;
    
    pub fn CreateBlockDeviceFile(newfile_hdl: *mut HandlePtr<FileHandle>, resolution_base: HandlePtr<FileHandle>, name: KStrCPtr, devid: Uuid, acl: HandlePtr<FileHandle>) -> SysResult;
    pub fn CreateCharDeviceFile(newfile_hdl: *mut HandlePtr<FileHandle>, resolution_base: HandlePtr<FileHandle>, name: KStrCPtr, devid: Uuid, acl: HandlePtr<FileHandle>) -> SysResult;

    pub fn CreateLegacyBlockDevice(newfile_hdl: *mut HandlePtr<FileHandle>, resolution_base: HandlePtr<FileHandle>, name: KStrCPtr, major: u32, minor: u32, acl: HandlePtr<FileHandle>) -> SysResult;
    pub fn CreateLegacyCharDevice(newfile_hdl: *mut HandlePtr<FileHandle>, resolution_base: HandlePtr<FileHandle>, name: KStrCPtr, major: u32, minor: u32, acl: HandlePtr<FileHandle>) -> SysResult;

    pub fn CreateNamedPipe(newfile_hdl: *mut HandlePtr<FileHandle>, acc: u32, resolution_base: HandlePtr<FileHandle>, name: KStrCPtr, acl: HandlePtr<FileHandle>) -> SysResult;

    pub fn CreateSymbolicLink(resolution_base: HandlePtr<FileHandle>, path: KStrCPtr, value: KStrCPtr) -> SysResult;

    pub fn ReadSymbolicLink(resolution_base: HandlePtr<FileHandle>, path: KStrCPtr, value_out: *mut KStrPtr) -> SysResult;

    pub fn CreateStream(stream_hdl: *mut HandlePtr<FileHandle>, file: HandlePtr<FileHandle>, stream_name: KStrCPtr, flags: u64) -> SysResult;
    pub fn RemoveStream(file: HandlePtr<FileHandle>, stream_name: KStrCPtr) -> SysResult;

    pub fn IsFileHandle(iohdl: HandlePtr<IOHandle>) -> SysResult;
}
