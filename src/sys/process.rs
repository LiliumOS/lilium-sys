use core::ffi::{c_long, c_ulong, c_void};
use core::mem::MaybeUninit;

use crate::uuid::parse_uuid;
use crate::{io::IOHandle, uuid::Uuid};

use super::kstr::KCSlice;
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

#[repr(C)]
pub struct ProcessStartContext {
    /// The base directory or `FileHandle` to open to spawn the process.
    ///
    /// If null, then it is an error to use relative paths.
    ///
    /// If `prg_path` is empty, then this file handle is read verbaitim.
    /// The file handle must be seekable (it must refer to a regular file or block device) in this case.
    ///
    pub prg_resolution_base: HandlePtr<FileHandle>,
    /// Path to the program to load
    ///
    /// If empty and `prg_resolution_base` is set, then `prg_resolution_base` is read as an executable from seek offset `0`.
    pub prg_path: KStrCPtr,
    /// Environment variables to initialize the process with
    pub environment: HandlePtr<EnvironmentMapHandle>,

    /// The flags to affect process spawning.
    /// The behaviour of each flag is documented on the flag.
    pub start_flags: c_long,
    /// Specifies the Security Context the process starts with.
    /// If this is null, then use the current security context of the calling thread
    ///
    /// If the spawned process is privilaged, that is, the program referred to by prg_path has either legacy unix SUID/SGID or an InstallSecurityContext stream,
    /// the behaviour differs:
    /// * If the start_security_context is null, then only the installed security context appears in the process, otherwise
    /// * The InstallSecurityContext stream (if any) obtains the base security context,
    ///   which is then merged with the start_security_context by adding in any permissions enabled or inheritible in the start_security context to the base security context, and revoking any that are revoked.
    /// * Then the primary and secondary principals are modified as follows: The Real Primary Principal is set to the one in start_security_context, and the Effective Primary principal is set to the same
    ///   only if the current thread has SET_PRIMARY_PRINCIPAL permission to the kernel. The Secondary Principal list is then populated by merging the two lists from start_security_context and from the installed context
    /// * Then if a the Legacy SUID bit is set the Primary Principal is set to the established principal and if SGID bit is set, then the secondary principal list is cleared and the estabblished secondary principal is added.
    /// * Non-revoked default permissions are then granted according to the Effective Primary Principal and Secondary Principals.
    ///
    /// Note that if a process has an InstallSecurityContext stream, the SUID/SGID bits in the LegacyUnixPermissions stream is ignored.
    /// If FLAG_NON_PRIVILAGED is set, then both the Legacy unix SUID/SGID bits and the InstallSecurityContext streams are ignored and the start_security_context is used as normal.
    ///
    /// If FLAG_REQUIRED_PRIVILAGED is set, then the before merging the permission set, the installed security context
    pub start_security_context: HandlePtr<SecurityContext>,
    /// The length of the array pointed to by `init_handles`
    pub init_handles_len: c_ulong,
    /// A pointer to an array of handles to pass into the spawned process.
    /// A corresponding array of such handles is given by the AT_PHANTOM_INIT_HANDLES array in the spawned process
    ///
    /// By convention, this array starts with the standard input, standard output, and standard error streams.
    pub init_handles: *const HandlePtr<Handle>,

    /// A program label, used for debugging or identifying the program via `EnumerateProcesses`
    pub label: KStrCPtr,

    /// Number of process arugments
    pub proc_args_len: c_ulong,
    /// Process arguments, including argv[0]
    pub proc_args: *const KStrCPtr,
    /// The namespace to place the process in
    pub init_namespace: HandlePtr<NamespaceHandle>,
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
/// [`MAP_ATTR_RESERVE`] cannot be used with [`MAP_KIND_ENCRYPTED`]. Note that because
pub const MAP_ATTR_RESERVE: u32 = 0x20;

pub const MAP_KIND_NORMAL: u32 = 0;
pub const MAP_KIND_RESIDENT: u32 = 1;
pub const MAP_KIND_SECURE: u32 = 2;
pub const MAP_KIND_ENCRYPTED: u32 = 3;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct MapExtendedAttrRaw {
    pub ty: Uuid,
    pub flags: u32,
    #[doc(hidden)]
    pub __pad: [u32; 3],
    pub data: [MaybeUninit<u8>; 32],
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct MapExtendedAttrBacking {
    #[doc(hidden)]
    pub __type_field: Uuid,
    pub flags: u32,
    #[doc(hidden)]
    pub __pad: [u32; 3],
    pub stream_base: u64,
    pub backing_file: HandlePtr<IOHandle>,
    #[doc(hidden)]
    pub __pad2: [MaybeUninit<u8>; 24 - core::mem::size_of::<HandlePtr<IOHandle>>()],
}

impl MapExtendedAttrBacking {
    pub const NULL: Self = Self {
        __type_field: parse_uuid("294d5c4e-cdf4-53b3-bfbc-ed804526394b"),
        flags: 0,
        stream_base: 0,
        backing_file: HandlePtr::null(),
        __pad: [0; 3],
        __pad2: [MaybeUninit::zeroed(); 24 - core::mem::size_of::<HandlePtr<IOHandle>>()],
    };
}

#[repr(C)]
pub union MapExtendedAttr {
    pub raw: MapExtendedAttrRaw,
    pub backing: MapExtendedAttrBacking,
}

#[repr(C)]
pub struct TerminationSignalInfo {
    pub signo: u32,
    pub is_thread_signal: bool,
}

#[allow(improper_ctypes)]
extern "C" {
    /// Obtains a handle to the current process environment
    pub fn GetCurrentEnvironment(hdl: *mut HandlePtr<EnvironmentMapHandle>) -> SysResult;
    /// Reads the given environment handle, with a given variable name, and stores the result in the KStr pointed to by `*val_out`
    pub fn GetEnvironmentVariable(
        hdl: HandlePtr<EnvironmentMapHandle>,
        name: KStrCPtr,
        val_out: *mut KStrPtr,
    ) -> SysResult;
    /// Writes the given environment handle, with a given variable name and value.
    pub fn SetEnvironmentVariable(
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
    ///
    /// `ctx` contains the information needed to spawn the process, including the program to run, the security context to start it with, and initial handles passed to the process
    pub fn CreateProcess(
        ctx: *const ProcessStartContext,
        hdl: *mut HandlePtr<ProcessHandle>,
    ) -> SysResult;

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
    /// If the process was terminated by a signal, returns SIGNALED and sets `*termsiginfo` to information about the signal that caused termination.
    ///
    /// If the process was terminated because the main thread was terminated by a call to `DestroyThread`, returns `KILLED`.
    ///
    ///
    pub fn JoinProcess(
        hdl: HandlePtr<ProcessHandle>,
        termsiginfo: *mut TerminationSignalInfo,
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
}
