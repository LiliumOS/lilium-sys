//! # Filesystem System Call Interfaces
//!  Interfaces for accessing the filesystem in Lilium
//!
//! ## Path Resolution
//!
//! The Lilium kernel implements two modes of path resolution: Logical and Physical.
//! The difference in behaviour is described in this section.
//!
//! By default, path names are resolved logically, exceptions are given.
//!
//! During Logical resolution, first each `.` component is removed, and each `..` component is removed with the last non-`.`/non-`..` path component, this results in navigating to the *logical* parent directory.
//! If any `..` components remain, they are resolved physical against the resolution base. If the component to be removed is `/`, then the `..` component is removed and the `/` component is kept.
//! After `..` components and `.` components are removed, the components are iterated through. If any symbolic links are encountered, they are resolved logically against the containing directory.
//!
//! Then finally, once symbolic links are traversed, the resulting path is then subject to physical resolution.
//!
//!
//! During physical resolution, path components, including `..` and `.` components are enumerated once at a time, and resolved against the filesystem object found by the previous directory.
//! Symbolic links are followed eagerly in physical resolution, and the symbol link content is resolved physically. A `..` that steps out of a symbolic link will reach the parent directory of the target,
//!  rather than the one of the symbolic link.
//!
//! The Limit to logical path resolution is 1024 segments separated by slashes (including a leading root segment `/`), including after reading symbolic links.
//!  If a path exceeds this length, either an error will be returned or the path will be resovled physical, either entirely or starting with the 1025th component. Which occurs is not specified.
//!
//! When resolving paths, the `DirectoryContent` stream is used by default (except for objects with type "SymbolicLink").
//! If path resolution of an object should be performed using an alternative stream, then after the name of the object, the name of the stream should be included in the component,
//! separated from the object name by `$$`.
//! If more than one stream with the name appears on the object, the first is used. To refer to any stream of that name other than the first, the number of the stream to be used should be referred to,
//!  separated from the stream name by `$`.
//! If the stream name would contain a `/`, then it should be escaped by a `\`.
//!
//! See the [Streams](#streams) section for more information on alternative streams.
//!
//! Most functions in the `fs` interface take a resolution base handle. If set to null, this uses the current directory. Otherwise, it must be a handle opened in `OP_DIRECTORY_ACCESS` mode, or an error occurs.
//! In both cases, the physical path is resolved physically against this, if it is relative (Does not start with a `/`)
//!
//! A path that starts with `//` (two adjacent separators) is reserved. This is a prefix designator - the root component follows the prefix designator and cannot be removed by `..` logical components
//!
//!

use core::{
    ffi::{c_long, c_ulong, c_void},
    mem::MaybeUninit,
};

use crate::uuid::Uuid;

use super::{
    handle::{Handle, HandlePtr},
    io::IOHandle,
    ipc::IPCServerHandle,
    kstr::{KCSlice, KSlice, KStrCPtr, KStrPtr},
    option::ExtendedOptionHead,
    result::SysResult,
    socket::SocketHandle,
};

/// A handle to a file on the filesytem
///
/// File handles are opened by `OpenFile`
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
/// Truncates the stream to size 0 after opening in `ACCESS_WRITE` mode
pub const ACCESS_TRUNCATE: u32 = 0x400;
/// Seek to the last byte of the file after opening. Has no effect if `ACCESS_TRUNCATE` is specified together with `ACCESS_WRITE`.
pub const ACCESS_START_END: u32 = 0x800;

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

pub use super::io::{MODE_ASYNC, MODE_BLOCKING, MODE_NONBLOCKING};

/// An option for opening the file
#[repr(C, align(32))]
#[derive(Copy, Clone)]
#[cfg_attr(feature = "bytemuck", derive(bytemuck::Zeroable))]
pub struct UnknownFileOpenOption {
    /// The header
    pub head: ExtendedOptionHead,
    /// The tail
    pub tail: [MaybeUninit<u8>; 64],
}

#[cfg(feature = "bytemuck")]
unsafe impl bytemuck::AnyBitPattern for UnknownFileOpenOption {}

#[repr(C, align(32))]
#[cfg_attr(feature = "bytemuck", derive(bytemuck::AnyBitPattern))]
#[derive(Copy, Clone)]
pub union FileOpenOption {
    /// The Header: Must be present on all subfields
    pub head: ExtendedOptionHead,
    /// Fallback type for all fields
    pub unknown: UnknownFileOpenOption,
}

#[repr(C)]
pub struct FileOpenOptions {
    /// If set to a non-empty string, designates the explicit stream of the object to open.
    ///
    /// Only certain filesystems support multiple streams (such as NTFS or LiliumFS). Other filesystems may only support a limited set of standard streams,
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
    /// Extended open options
    pub extended_options: KCSlice<FileOpenOption>,
}

#[repr(C)]
pub struct DirectoryInfo {
    pub fname: KStrPtr,
    pub flags: u64,
    pub acl_handle: HandlePtr<FileHandle>,
}

#[repr(C)]
pub struct ReadDaclRow {
    pub applied: Uuid,
    pub stream_name: KStrPtr,
    pub perm_name: KStrPtr,
    pub principal: Uuid,
    pub mode: u32,
}

#[repr(C)]
pub struct DaclRow {
    pub applied: Uuid,
    pub stream_name: KStrCPtr,
    pub perm_name: KStrCPtr,
    pub principal: Uuid,
    pub mode: u32,
}

pub const ACL_MODE_ALLOW: u32 = 0;
pub const ACL_MODE_DENY: u32 = 1;
pub const ACL_MODE_FORBID: u32 = 2;
pub const ACL_MODE_INHERIT: u32 = 3;

#[cfg(any(feature = "io", doc))]
#[expect(improper_ctypes)]
unsafe extern "system" {
    /// Opens a new file handle with the given path
    pub fn OpenFile(
        hdl: *mut HandlePtr<FileHandle>,
        resolution_base: HandlePtr<FileHandle>,
        path: KStrCPtr,
        opts: *const FileOpenOptions,
    ) -> SysResult;

    /// Reopens the given file, allowing you to change access modes and operation modes. The handle is modified in-place - this affects any `SharedHandle` created from it or any handle recieved on another thread
    /// If an error occurs, the hdl continues to refer to the original object or stream in the previous operating mode.
    ///
    /// This operation does not change the referent of the handle, in particular:
    /// * If hdl was open with a stream override (IE. it does not refer to the whole object), then after this call, it will continue to refer to that stream - if `stream_override` is present in `new_opts`, PERMISSION is returned.
    /// * If hdl referred to a symlink, then it will continue to refer to that symlink, whether or not`ACCESS_LINK_STREAM_DIRECT` is present in `new_opts`
    /// * paths are not resolved when reopening the file
    pub fn ReopenFile(hdl: HandlePtr<FileHandle>, new_opts: *const FileOpenOptions) -> SysResult;

    pub fn DuplicateFile(
        new_hdl: *mut HandlePtr<FileHandle>,
        old_hdl: HandlePtr<FileHandle>,
    ) -> SysResult;

    pub fn CloseFile(hdl: HandlePtr<FileHandle>) -> SysResult;

    /// Advances the directory iterator
    pub fn DirectoryNext(hdl: HandlePtr<FileHandle>, state: *mut *mut c_void) -> SysResult;
    pub fn DirectoryStep(
        hdl: HandlePtr<FileHandle>,
        state: *mut *mut c_void,
        num: c_ulong,
    ) -> SysResult;
    pub fn DirectoryRead(
        hdl: HandlePtr<FileHandle>,
        state: *mut c_void,
        info: *mut DirectoryInfo,
    ) -> SysResult;
    pub fn DirectoryReadMany(
        hdl: HandlePtr<FileHandle>,
        state: *mut c_void,
        info: *mut KSlice<DirectoryInfo>,
    ) -> SysResult;

    pub fn StreamSize(hdl: HandlePtr<FileHandle>) -> SysResult;
    pub fn ObjectSize(hdl: HandlePtr<FileHandle>, size_out: *mut u128) -> SysResult;

    pub fn CreateAcl(hdl: *mut HandlePtr<FileHandle>) -> SysResult;
    pub fn DefaultAcl(hdl: *mut HandlePtr<FileHandle>) -> SysResult;
    pub fn ObjectOwner(hdl: HandlePtr<FileHandle>, uuid: *mut Uuid) -> SysResult;
    pub fn SetObjectOwner(hdl: HandlePtr<FileHandle>, uuid: *const Uuid) -> SysResult;
    pub fn AclNextRow(hdl: HandlePtr<FileHandle>, state: *mut *mut c_void) -> SysResult;
    pub fn AclReadRow(
        hdl: HandlePtr<FileHandle>,
        state: *mut c_void,
        info: *mut ReadDaclRow,
    ) -> SysResult;
    pub fn AclWriteRow(
        hdl: HandlePtr<FileHandle>,
        state: *mut c_void,
        new_row: *const DaclRow,
    ) -> SysResult;
    pub fn AclLegacyUid(hdl: HandlePtr<FileHandle>) -> SysResult;
    pub fn AclLegacyGid(hdl: HandlePtr<FileHandle>) -> SysResult;
    pub fn AclLegacyMode(hdl: HandlePtr<FileHandle>) -> SysResult;
    pub fn AclSetLegacyMode(hdl: HandlePtr<FileHandle>, mode: u32) -> SysResult;
    pub fn AclSetLegacyUid(hdl: HandlePtr<FileHandle>, uid: c_long) -> SysResult;
    pub fn AclSetLegacyGid(hdl: HandlePtr<FileHandle>, gid: c_long) -> SysResult;
    pub fn OverwriteAcl(
        file_hdl: HandlePtr<FileHandle>,
        acl_hdl: HandlePtr<FileHandle>,
    ) -> SysResult;
    pub fn CopyAcl(
        acl_out: *mut HandlePtr<FileHandle>,
        file_hdl: HandlePtr<FileHandle>,
    ) -> SysResult;
    pub fn SetDefaultAcl(acl: HandlePtr<FileHandle>) -> SysResult;
    pub fn AclTestPermission(
        hdl: HandlePtr<FileHandle>,
        permission: KStrCPtr,
        stream: KStrCPtr,
    ) -> SysResult;
    pub fn AclSetPermission(
        hdl: HandlePtr<FileHandle>,
        permission: KStrCPtr,
        stream: KStrCPtr,
        id: &Uuid,
        mode: u32,
    ) -> SysResult;

    pub fn CreateDirectory(
        dir_handle: *mut HandlePtr<FileHandle>,
        resolution_base: HandlePtr<FileHandle>,
        path: KStrCPtr,
        acl: HandlePtr<FileHandle>,
    ) -> SysResult;

    /// Creates and opens a directory in an existing directory that does not alias with any other filesystem object, and cannot be named by any other thread.
    /// `resolutionbase` determines the behaviour of path traversals that use `..` to resolve directories outside of the created directory. It also may be used to determine the filesystem the object is created on
    /// If no `resolutionbase` is given, the current resolution base directory is used instead.
    ///
    /// The directory and its contents will be unlinked when the last handle to it is closed. To preserve the contents, `AssociateName` must be used.
    /// This is guaranteed to succeed only if `resolutionbase` and the `new_name_base` of the `AssociateName` call belong to the same filesystem.
    ///
    /// ## Notes
    /// Privacy is ensured on a best-effort basis only, and the degree to which it is provided is filesystem dependent.
    /// LiliumFS guarantees that no other thread can open the object created this way,
    ///  but other filesystems may permit a racing thread to observe the directory under an unspecified name before it is removed from path resolution.
    /// Virtual Filesystems can provide the same privacy guarantees if they provide an implementation for `fs_create_anonymous_object`.
    ///
    /// Regardless of these limitations, the aliasing rule is guaranteed to be upheld, and this function will never error because any object with a given name already exists in the resolution base.
    ///
    /// Private directories created by this function are thread-local
    ///
    /// ## Errors
    ///
    /// Returns PERMISSION if the kernel perission `CREATE_ANONYMOUS_OBJECT` is denied to the current thread, or `CREATE_OBJECT` ACL permission is denied to
    ///  `resolutionbase`.
    ///
    /// Returns RESOURCE_LIMIT_EXHAUSTED if the kernel resource limit `ANONYMOUS_OBJECT_MAX` is exceeded by the current security context.
    ///
    /// Returns BUSY if the object cannot be created due to a pendng device operation on the filesystem.
    ///
    /// Returns DEVICE_FULL if the object cannot be created due to size limits on the filesystem.
    ///
    /// Returns UNSUPPORTED_OPERATION is the filesystem does not support anonymous directories.
    ///
    /// Returns `INVALID_OPTION` if any extended option specified by `options` is invalid.
    ///
    pub fn CreatePrivateDirectory(
        dir_handle: *mut HandlePtr<FileHandle>,
        resolutionbase: HandlePtr<FileHandle>,
        acl: HandlePtr<FileHandle>,
        options: *const KCSlice<FileOpenOption>,
    ) -> SysResult;

    pub fn CreateHardLink(
        newfile_hdl: *mut HandlePtr<FileHandle>,
        new_name_base: HandlePtr<FileHandle>,
        new_name: KStrCPtr,
        old_name_base: HandlePtr<FileHandle>,
        old_name: KStrCPtr,
    ) -> SysResult;
    /// Associates the given object with a name specified by `new_name` beginning resolution from `new_name_base`. This is equivalent to `CreateHardLink` but refers to an object by handle rather than by path.
    ///
    /// This function may be used if the object has no name (IE. a directory created via `CreatePrivateDirectory`),
    ///
    pub fn AssociateName(
        file: HandlePtr<FileHandle>,
        new_name_base: HandlePtr<FileHandle>,
        new_name: KStrCPtr,
    ) -> SysResult;
    pub fn CreateWeakLink(
        newfile_hdl: *mut HandlePtr<FileHandle>,
        new_name_base: HandlePtr<FileHandle>,
        new_name: KStrCPtr,
        old_name_base: HandlePtr<FileHandle>,
        old_name: KStrCPtr,
    ) -> SysResult;
    pub fn AssociateWeakName(
        file: HandlePtr<FileHandle>,
        new_name_base: HandlePtr<FileHandle>,
        new_name: KStrCPtr,
    ) -> SysResult;

    pub fn UpgradeLink(
        newfile_hdl: *mut HandlePtr<FileHandle>,
        resolution_base: HandlePtr<FileHandle>,
        name: KStrCPtr,
    ) -> SysResult;
    pub fn DowngradeLink(resolution_base: HandlePtr<FileHandle>, name: KStrCPtr) -> SysResult;

    pub fn RenameObject(
        new_name_base: HandlePtr<FileHandle>,
        new_name: KStrCPtr,
        old_name_base: HandlePtr<FileHandle>,
        old_name: KStrCPtr,
    ) -> SysResult;
    pub fn RemoveLink(resolution_base: HandlePtr<FileHandle>, path: KStrCPtr) -> SysResult;

    pub fn CreateNamedChannel(
        ipc_hdl: HandlePtr<IPCServerHandle>,
        name_base: HandlePtr<FileHandle>,
        name: KStrCPtr,
        acl: HandlePtr<FileHandle>,
    ) -> SysResult;
    pub fn CreateNamedServer(
        sock_hdl: HandlePtr<SocketHandle>,
        name_base: HandlePtr<FileHandle>,
        name: KStrCPtr,
        acl: HandlePtr<FileHandle>,
    ) -> SysResult;

    pub fn CreateBlockDeviceFile(
        newfile_hdl: *mut HandlePtr<FileHandle>,
        resolution_base: HandlePtr<FileHandle>,
        name: KStrCPtr,
        devid: Uuid,
        acl: HandlePtr<FileHandle>,
    ) -> SysResult;
    pub fn CreateCharDeviceFile(
        newfile_hdl: *mut HandlePtr<FileHandle>,
        resolution_base: HandlePtr<FileHandle>,
        name: KStrCPtr,
        devid: Uuid,
        acl: HandlePtr<FileHandle>,
    ) -> SysResult;

    pub fn CreateLegacyBlockDevice(
        newfile_hdl: *mut HandlePtr<FileHandle>,
        resolution_base: HandlePtr<FileHandle>,
        name: KStrCPtr,
        major: u32,
        minor: u32,
        acl: HandlePtr<FileHandle>,
    ) -> SysResult;
    pub fn CreateLegacyCharDevice(
        newfile_hdl: *mut HandlePtr<FileHandle>,
        resolution_base: HandlePtr<FileHandle>,
        name: KStrCPtr,
        major: u32,
        minor: u32,
        acl: HandlePtr<FileHandle>,
    ) -> SysResult;

    pub fn CreateNamedPipe(
        newfile_hdl: *mut HandlePtr<FileHandle>,
        acc: u32,
        resolution_base: HandlePtr<FileHandle>,
        name: KStrCPtr,
        acl: HandlePtr<FileHandle>,
    ) -> SysResult;

    pub fn CreateSymbolicLink(
        resolution_base: HandlePtr<FileHandle>,
        path: KStrCPtr,
        value: KStrCPtr,
    ) -> SysResult;

    pub fn ReadSymbolicLink(
        resolution_base: HandlePtr<FileHandle>,
        path: KStrCPtr,
        value_out: *mut KStrPtr,
    ) -> SysResult;

    pub fn CreateStream(
        stream_hdl: *mut HandlePtr<FileHandle>,
        file: HandlePtr<FileHandle>,
        stream_name: KStrCPtr,
        flags: u64,
    ) -> SysResult;
    pub fn RemoveStream(file: HandlePtr<FileHandle>, stream_name: KStrCPtr) -> SysResult;

    pub fn IsFileHandle(iohdl: HandlePtr<IOHandle>) -> SysResult;

    pub fn GetObjectType(file: HandlePtr<FileHandle>) -> SysResult;

    /// Changes the access and (optionally) the operation mode of a file handle, without reopening the underlying object or stream.
    /// This operation does not modify the existing handle, and instead places a new handle opened in the new access mode and operation mode in `newhdl`
    ///
    /// Path resolution is not performed again, but permission checks reoccurs if:
    /// * The operation mode is changed from the old handle, or
    /// * The new access mode has any modes not present in the access mode of the old handle.
    ///
    /// Note that permission checks will still occur even if a change occurs for which the permissions are subsumed by the old handle (for example, setting an op mode of `OP_NO_ACCESS`,
    ///     or adding only `ACCESS_LOCK_SOFT`, or `ACCESS_CREATE` which is only effective on `OpenFile`).
    ///
    /// Setting `new_op` to `0` does not alter the operation mode.
    ///
    /// Changing the lock mode of the handle is not possible via `ChangeFileAccessMode`, except that a new lock may be established
    pub fn ChangeFileAccessMode(
        newhdl: *mut HandlePtr<FileHandle>,
        oldhdl: HandlePtr<FileHandle>,
        new_access: u32,
        new_op: u32,
    ) -> SysResult;

    pub fn SetCurrentDirectory(dir: HandlePtr<FileHandle>) -> SysResult;

    pub fn SetResolutionRoot(res_base: HandlePtr<FileHandle>) -> SysResult;

    pub fn CopyAllStreams(
        old_file: HandlePtr<FileHandle>,
        new_file: HandlePtr<FileHandle>,
        total_size_out: *mut u128,
    ) -> SysResult;
}
