use core::ffi::{c_long, c_ulong, c_void};
use core::mem::MaybeUninit;

use crate::uuid::parse_uuid;
use crate::{sys::io::IOHandle, uuid::Uuid};

use super::except::ExceptionStatusInfo;
use super::kstr::KCSlice;
use super::option::ExtendedOptionHead;
use super::{
    fs::FileHandle,
    handle::{Handle, HandlePtr},
    isolation::NamespaceHandle,
    kstr::{KStrCPtr, KStrPtr},
    permission::SecurityContext,
    result::SysResult,
};

#[repr(transparent)]
pub struct ProcessHandle(Handle);

/// Place the program in a SUSPENDED state before spawning. The current thread may thus attach a debugger and debug the process immediately without sending a DebugSuspend
pub const FLAG_START_SUSPENDED: c_long = 0x02;
/// Ignore Legacy UNIX SUID/SGID or InstallSecurityContext filesystem requirements, and guarantee start as the supplied security context
pub const FLAG_NON_PRIVILAGED: c_long = 0x04;
/// Require at least the privilages in start_security_context when spawning a privilaged process
pub const FLAG_REQUIRED_PRIVILAGED: c_long = 0x08;
/// Sets `PROC_ATTR_HIDDEN` on the created process
pub const FLAG_HIDE_PROCESS: c_long = 0x10;
/// Does not load process using standard interpreter loading rules (#! or ELF `.interp` segment).
///
/// Note that if the program has a dynamic section, it will not be loaded correctly. Thus, this is a dangerous flag to use.
///
/// It is an error to apply this to a privilaged process (one that has an InstallSecurityContext stream or legacy unix SUID/SGID), unless the current thread has the NoInterpPrivilaged kernel permission
pub const FLAG_NO_INTERP: c_long = 0x20;

/// Loads the given program into the current process, destroying the current image running in the process
/// The [`CreateProcess`] syscall will not return succesfully, and any thread is terminated as though by [`DestroyThread`]
pub const FLAG_REPLACE_IMAGE: c_long = 0x40;

/// Fallback type for [`CreateProcessOption`]
#[repr(C, align(32))]
#[derive(Copy, Clone)]
#[cfg_attr(feature = "bytemuck", bytemuck::Zeroable, bytemuck::AnyBitPattern)]
pub struct CreateProcessOptionRaw {
    /// The Header of the opton.
    pub header: ExtendedOptionHead,
    /// The content of the option.
    pub data: [MaybeUninit<u8>; 32],
}

#[repr(C, align(32))]
#[derive(Copy, Clone)]
pub struct CreateProcessOptionInitHandles {
    /// The Header of the opton.
    pub header: ExtendedOptionHead,
    /// THe array of handles to pass to the process.
    ///
    /// The handle pointers are not directly copied into the initial threads handle list, but instead new handles that refer to the same object with the same capabilities are created in the initial thread.
    /// These handles are placed in an array pointed to by [`AT_LILIUM_INIT_HANDLES`][super::elf::AT_LILIUM_INIT_HANDLES], and the length of the array is given in [`AT_LILIUM_INIT_HANDLES_LEN`][super::elf::AT_LILIUM_INIT_HANDLES].
    ///
    /// By convention, the first 3 items are [`IOHandle`]s that become the values of the inital threads [`__HANDLE_IO_STDIN`][super::io::__HANDLE_IO_STDIN], [`__HANDLE_IO_STDOUT`][super::io::__HANDLE_IO_STDOUT],
    ///  and [`__HANDLE_IO_STDERR`][super::io::__HANDLE_IO_STDERR].
    ///
    /// If the option is not specified, it acts as though handles to the process's default stdin/stdout/stderr objects are passed as the first 3 elements,
    /// provided the appropriate handle has not been closed on the current thread (in this case, the corresponding handle is set to `null`)
    /// Note that the default objects are not guaranteed to match the handles (as the handles may be overwritten by the process),
    ///  the default objects are taken from the first 3 elements of the init_handles array when the process is spawned, if the array is sufficiently sized (otherwise the default objects are set to null).
    pub init_handles: KCSlice<HandlePtr<Handle>>,
}

#[repr(C, align(32))]
#[derive(Copy, Clone)]
pub struct CreateProcessOptionEnvironment {
    /// The header of the option.
    pub header: ExtendedOptionHead,
    /// A handle to the environment map to spawn the process with.
    ///
    /// If the option is omitted, the handle will refer to a copy of the environment of the current thread.
    pub environment: HandlePtr<EnvironmentMapHandle>,
}

#[repr(C, align(32))]
#[derive(Copy, Clone)]
pub struct CreateProcessOptionArgs {
    /// The header of the option.
    pub header: ExtendedOptionHead,
    /// The List of Arguments to pass to the process
    pub arguments: KCSlice<KStrCPtr>,
}

#[repr(C, align(32))]
#[derive(Copy, Clone)]
#[cfg_attr(feature = "bytemuck", bytemuck::Zeroable, bytemuck::AnyBitPattern)]
pub union CreateProcessOption {
    /// The Header
    pub head: ExtendedOptionHead,
    /// Fallback field for statically unknown options
    pub raw: CreateProcessOptionRaw,
    /// Specifies the initial handles array for the spawned process
    pub init_handles: CreateProcessOptionInitHandles,
    /// Specifies the environment map
    pub env_map: CreateProcessOptionEnvironment,
}

#[repr(transparent)]
pub struct EnvironmentMapHandle(Handle);

/// View processs spawned with the `FLAG_HIDE_PROCESS` flag.
pub const ENUMERATE_VIEW_HIDDEN: u32 = 0x01;
/// View all processes, not just the ones that match the current primary principal.
/// Requires the `ViewAllProcesses` kernel permisson
pub const ENUMERATE_VIEW_ALL: u32 = 0x02;

/// `EnumerateRead` does not fail if it attempts to access a process that the current thread does not have `AccessProcess` permission to.
/// In such cases, the process handle stored to the `ProcessInfo` struct is `null`
pub const ENUMERATE_NO_FAIL: u32 = 0x04;

#[repr(transparent)]
pub struct EnumerateProcessHandle(Handle);

#[repr(C)]
pub struct ProcessInfo {
    /// The UUID of the primary principal the process was spawned with
    pub primary_principal: Uuid,
    /// Same as `primary_principal`, but takes into account the `InstallSecurityContext` stream and legacy unix SUID/SGID
    pub effective_primary_principal: Uuid,
    /// The handle to the process read
    pub handle: HandlePtr<ProcessHandle>,
    /// The process label - set to the same as the label in `ProcessStartContext`
    pub label: KStrPtr,
    /// The executable name - specified in ther first element of the `proc_args` array
    pub exec_name: KStrPtr,
    /// The full path to the program
    pub prg_path: KStrPtr,
}

/// Creates a Readable mapping.
///
/// Generally, a non-readable mapping cannot be created (without setting [`MAP_ATTR_RESERVE`]).
/// The exception is that some architectures may permit write-only or execute-only pages.
pub const MAP_ATTR_READ: u32 = 0x01;
///
pub const MAP_ATTR_WRITE: u32 = 0x02;
pub const MAP_ATTR_EXEC: u32 = 0x04;
pub const MAP_ATTR_THREAD_PRIVATE: u32 = 0x08;
pub const MAP_ATTR_PROC_PRIVATE: u32 = 0x10;
/// Reserves (but does not allocate new physical memory or swap for) the mapping region.
/// This allows you to preallocate a region of memory without consuming either proper memory or the security context memory resource limit (beyond memory consumed for kernel-level data structures), until actually needed.
///
///
/// Regardless of [`MAP_ATTR_READ`], [`MAP_ATTR_WRITE`], or [`MAP_ATTR_EXEC`], accesses to this region will fault (and generally terminate execution unless the program has arranged to handle the exception)
///
/// Like any other mapping, a mapping created with [`CreateMapping`] using [`MAP_ATTR_RESERVE`] will be an invalid region for other calls to [`CreateMapping`].
/// To make use of part of the reserved region, use [`ChangeMappingAttributes`] on the relevant part.
///
/// [`MAP_ATTR_RESERVE`] cannot be used with [`MAP_KIND_ENCRYPTED`].
pub const MAP_ATTR_RESERVE: u32 = 0x20;

pub const MAP_KIND_NORMAL: u32 = 0;
pub const MAP_KIND_RESIDENT: u32 = 1;
pub const MAP_KIND_SECURE: u32 = 2;
pub const MAP_KIND_ENCRYPTED: u32 = 3;

#[repr(C, align(32))]
#[derive(Copy, Clone)]
#[cfg_attr(feature = "bytemuck", bytemuck::Zeroable, bytemuck::AnyBitPattern)]
pub struct MapExtendedAttrRaw {
    pub header: ExtendedOptionHead,
    pub data: [MaybeUninit<u8>; 32],
}

#[repr(C, align(32))]
#[derive(Copy, Clone)]
pub struct MapExtendedAttrBacking {
    pub header: ExtendedOptionHead,
    pub stream_base: u64,
    pub backing_file: HandlePtr<IOHandle>,
}

impl MapExtendedAttrBacking {
    pub const NULL: Self = Self {
        header: ExtendedOptionHead {
            ty: parse_uuid("294d5c4e-cdf4-53b3-bfbc-ed804526394b"),
            ..ExtendedOptionHead::ZERO
        },
        stream_base: 0,
        backing_file: HandlePtr::null(),
    };
}

#[repr(C, align(32))]
#[derive(Copy, Clone)]
pub struct MapExtendedAttrName {
    pub header: ExtendedOptionHead,
    pub mapping_name: KStrCPtr,
}

impl MapExtendedAttrName {
    pub const NULL: Self = Self {
        header: ExtendedOptionHead {
            ty: parse_uuid("90ded1b2-ba85-5d34-90b4-74e717444863"),
            ..ExtendedOptionHead::ZERO
        },
        mapping_name: KStrCPtr::empty(),
    };
}

#[repr(C, align(32))]
#[derive(Copy, Clone)]
#[cfg_attr(feature = "bytemuck", bytemuck::Zeroable, bytemuck::AnyBitPattern)]
pub union MapExtendedAttr {
    pub raw: MapExtendedAttrRaw,
    pub backing: MapExtendedAttrBacking,
    pub mapping_name: MapExtendedAttrName,
}

#[expect(improper_ctypes)]
unsafe extern "system" {
    /// Obtains a handle to the current process environment
    pub unsafe fn GetCurrentEnvironment(hdl: *mut HandlePtr<EnvironmentMapHandle>) -> SysResult;
    /// Reads the given environment handle, with a given variable name, and stores the result in the KStr pointed to by `*val_out`
    pub unsafe fn GetEnvironmentVariable(
        hdl: HandlePtr<EnvironmentMapHandle>,
        name: KStrCPtr,
        val_out: *mut KStrPtr,
    ) -> SysResult;
    /// Writes the given environment handle, with a given variable name and value.
    pub unsafe fn SetEnvironmentVariable(
        hdl: HandlePtr<EnvironmentMapHandle>,
        name: KStrCPtr,
        val: KStrCPtr,
    ) -> SysResult;

    /// Creates an empty environment mpa
    pub fn CreateEnvironment(hdl: *mut HandlePtr<EnvironmentMapHandle>) -> SysResult;

    /// Copies the given environment mpa
    pub fn CopyEnvironment(
        hdl: *mut HandlePtr<EnvironmentMapHandle>,
        map: HandlePtr<EnvironmentMapHandle>,
    ) -> SysResult;

    /// Enumerates over the list of key-value pairs in the environment map
    pub fn EnvironmentNextPair(
        hdl: HandlePtr<EnvironmentMapHandle>,
        state: *mut *mut c_void,
    ) -> SysResult;

    /// Reads the current key-value pair in the enumerate
    pub fn EnvironmentReadPair(
        hdl: HandlePtr<EnvironmentMapHandle>,
        state: *mut c_void,
    ) -> SysResult;

    /// Spawns a new process and places a handle to it in `hdl`.
    ///
    /// ## Parameters
    ///
    /// `resolution_base` and `path` are used to find the executable object as follows:
    /// * If `path` is an absolute path (starts with `/`), then `resolution_base` is ignored, and the executable object is located by resolving `path`.
    /// * If `path` is a non-empty relative path, it it resolved relative to `resolution_base` or the thread's current working directory if `resolution_base` is null,
    /// * If `path` is empty, then the executable object is given by `resolution_base` directly and path resolution is not performed.
    ///
    /// ## Object Lookup
    ///
    /// When `resolution_base` is used to directly access the object, it is not resolved as a symbolic link. It must refer to a data stream (such as `Data`).
    /// The handle must have [`io::CHAR_READABLE`][super::io::CHAR_READABLE] and [io::CHAR_SEEKABLE][super::io::CHAR_SEEKABLE]. Regardless of characters and capabilities, a DACL permission check for the `Executable` permission.
    ///
    /// When `resolution_base` is used to resolve a path, the handle must have the capability `SearchDirectory` and must be valid for path resolution.
    /// Symbolic link streams are resolved using physical path resolution before resollving `path` using symbolic path resolution.
    ///
    /// When a new object is opened, symbolic links are resolved, then the `Data` stream is opened if defined, regardless of the object type.
    /// The `Executable` DACL permission is checked, and the `Readable` DACL permission is ignored for this stream open
    /// Otherwise, if a default stream type that is not recognized as a `Data` Stream, an [`INVALID_OPERATION`][super::result::errors::INVALID_OPERATION] error is returned.
    ///
    /// ## Image Loading
    ///
    /// Unless [`FLAG_NO_INTERP`] is set, the process image is loaded in two phases:
    /// * First, if the image is an ELF Executable* file that is appropriate for the architecture, then it is mapped into memory as though each `PT_LOAD` segment is used in a [`CreateMapping`] call
    /// * The interpreter for the image is then determined:
    ///     * If it is a ELF File, the content of the `PT_INTERP` segment, if any, is read and interpreted as a path, resolved by `resolution_base` if it is relative.
    ///     * If the file starts with an optional UTF-8 BOM, then the strings `#!!` or `#!` in ASCII followed by valid UTF-8 up to and including the first occurance of a `0x0a` byte in the file,
    ///   the interpreter is read from the remainder of the `\n` terminated line, resolved by `resolution_base` if it is relative.
    ///     * Otherwise, if the kernel supports custom format images, the supplier object is returned from a device command `17494569-c542-51f6-bf30-7154703b7f79` (ResolveImageInterpreter)
    ///  issued to the custom image format resolver device, with a handle to the object that has [`io::CHAR_READABLE`][super::io::CHAR_READABLE] and no capabilities.
    /// The command returns a `HandlePtr` to a `FileHandle` that points to the interpreter.
    ///
    ///
    pub fn CreateProcess(
        hdl: *mut HandlePtr<ProcessHandle>,
        resolution_base: HandlePtr<FileHandle>,
        path: *const KStrCPtr,
        options: *const KCSlice<CreateProcessOption>,
    ) -> SysResult;

    /// Causes the process designated by `hdl` to terminate, as though it recieved an unmanaged exception with code `79a90b8e-8f4b-5134-8aa2-ff68877017db` (RemoteStop)
    ///
    ///
    pub fn TerminateProcess(hdl: HandlePtr<ProcessHandle>) -> SysResult;

    ///
    /// Enumerates over the list of processes on the system
    pub fn EnumerateProcesses(hdl: *mut HandlePtr<EnumerateProcessHandle>, flags: u32)
        -> SysResult;

    /// Advances the enumeration list
    pub fn EnumerateNextProc(
        hdl: HandlePtr<EnumerateProcessHandle>,
        state: *mut *mut c_void,
    ) -> SysResult;

    /// Reads from the current pointer in the EnumerateProcessHandle
    pub fn EnumerateReadProc(
        hdl: HandlePtr<EnumerateProcessHandle>,
        state: *mut c_void,
        info: *mut ProcessInfo,
    ) -> SysResult;

    /// Waits for the given process. The current thread is blocked until it completes.
    ///
    /// A return of a value described below syncronizes-with the termination of all threads running in that process
    ///
    /// If the process is terminated by a call to `ExitProcess` or by `ExitThread` called from the main thread,
    ///  returns that value exactly.
    ///
    /// If the process was terminated by an unmanaged exception, returns SIGNALED and sets `*termsiginfo` to information about the exception that caused the termination.
    ///
    /// If the process was terminated because the main thread was terminated by a call to `DestroyThread`, returns `KILLED`.
    ///
    ///
    pub fn JoinProcess(
        hdl: HandlePtr<ProcessHandle>,
        termsiginfo: *mut ExceptionStatusInfo,
    ) -> SysResult;

    /// Detaches the given process from the handle
    pub fn DetachProcess(hdl: HandlePtr<ProcessHandle>) -> SysResult;

    /// Terminates all threads as though by `DestroyThread` syscalls, and exits from the process with the given code
    ///
    /// The termination of other threads occurs at such a time as the thread might recieve a signal from `SignalThread`.
    pub fn ExitProcess(code: u32) -> !;

    /// Creates a new Mapping at `base_addr` if possible (if `0`, picks an address and returns it in `base_addr`), of length `page_count`, using the specified kind and attributes.
    ///
    ///
    ///
    pub fn CreateMapping(
        base_addr: *mut *mut c_void,
        page_count: c_long,
        map_attrs: u32,
        map_kind: u32,
        map_ext: *const KCSlice<MapExtendedAttr>,
    ) -> SysResult;

    /// Changes the attributes of (part)
    pub fn ChangeMappingAttributes(
        mapping_base_addr: *mut c_void,
        page_count: c_long,
        new_map_attrs: u32,
    ) -> SysResult;

    pub fn RemoveMapping(mapping_base_addr: *mut c_void, page_count: c_long) -> SysResult;

    pub fn ResizeMapping(
        mapping_base_addr: *mut c_void,
        old_page_count: c_long,
        new_base_addr: *mut c_void,
        new_page_count: c_long,
    ) -> SysResult;

    /// Aborts the process. This will first call [`ExceptHandleSynchronous`][crate::sys::except::ExceptHandleSynchronous] with exception `466fbae6-be8b-5525-bd04-ee7153b74f55` (ProcessAbort),
    ///  then calls [`UnmangedException`][crate::sys::except::UnmanagedException] after USI registered handlers (including those registered via `signal`) return.
    pub safe fn abort() -> !;
}
