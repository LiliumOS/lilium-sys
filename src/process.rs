use core::ffi::c_void;
use core::{
    ffi::{c_long, c_ulong},
    marker::PhantomData,
    mem::MaybeUninit,
    ops::Deref,
};

#[cfg(debug_assertions)]
#[track_caller]
#[inline(always)]
#[cold]
unsafe fn debug_unreachable() -> ! {
    unreachable!()
}

#[cfg(not(debug_assertions))]
#[inline(always)]
unsafe fn debug_unreachable() -> ! {
    core::hint::unreachable_unchecked()
}

use alloc::{
    string::{String, ToString},
    vec::Vec,
};

use crate::{
    fs::{Path, PathBuf},
    handle::{AsHandle, BorrowedHandle},
    io::IOHandle,
    result::Result,
    security::SecurityContext,
    sys::{
        fs::FileHandle,
        handle::{Handle, HandlePtr},
        io::{__HANDLE_IO_STDERR, __HANDLE_IO_STDIN, __HANDLE_IO_STDOUT},
        isolation::NamespaceHandle,
        kstr::KStrCPtr,
        process::{
            self as sys, CreateProcess, EnumerateProcessHandle, EnvironmentMapHandle,
            ProcessHandle, ProcessStartContext,
        },
    },
};

bitflags::bitflags! {
    pub struct ProcessStartFlags : c_long{
        const START_SUSPENDED = sys::FLAG_START_SUSPENDED;
        const NON_PRIVLAGED = sys::FLAG_NON_PRIVILAGED;
        const REQUIRE_PRIVILAGED = sys::FLAG_REQUIRED_PRIVILAGED;
        const HIDE_PROCESS = sys::FLAG_HIDE_PROCESS;
        const NO_INTERP = sys::FLAG_NO_INTERP;
        const REPLACE_IMAGE = sys::FLAG_REPLACE_IMAGE;
    }
}

pub struct Command<'a> {
    resolution_base: HandlePtr<FileHandle>,
    cmd: PathBuf,
    env: HandlePtr<EnvironmentMapHandle>,
    namespace: HandlePtr<NamespaceHandle>,
    start_security_context: HandlePtr<SecurityContext>,
    args: Vec<String>,
    init_handles: Vec<HandlePtr<Handle>>,
    label: String,
    flags: ProcessStartFlags,
    _handles: PhantomData<BorrowedHandle<'a, Handle>>,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum CommandStatus {
    Normal(i32),
    Abnormal(u32),
    Killed,
}

struct CommandResult {
    hdl: HandlePtr<ProcessHandle>,
}

impl CommandResult {
    fn join(self) -> crate::result::Result<CommandStatus> {
        let mut sigterminfo = MaybeUninit::uninit();
        let ret = unsafe { crate::sys::process::JoinProcess(self.hdl, sigterminfo.as_mut_ptr()) };
        loop {
            match crate::result::Error::from_code(ret) {
                Ok(()) => break Ok(CommandStatus::Normal(ret as i32)), // Note: Lilium guarantees it will be a positive i32
                Err(crate::result::Error::Killed) => break Ok(CommandStatus::Killed),
                Err(crate::result::Error::Signaled) => {
                    break Ok(CommandStatus::Abnormal(
                        unsafe { sigterminfo.assume_init() }.signo,
                    ))
                }
                Err(crate::result::Error::Interrupted) => continue,
                Err(crate::result::Error::Timeout) => {
                    unsafe { crate::sys::thread::ClearBlockingTimeout() };
                    continue;
                }
                Err(e) => break Err(e),
            }
        }
    }

    fn detach(self) -> crate::result::Result<()> {
        crate::result::Error::from_code(unsafe { crate::sys::process::DetachProcess(self.hdl) })
    }
}

impl Command<'_> {
    fn spawn_with_result(&mut self) -> crate::result::Result<CommandResult> {
        let proc_args = self
            .args
            .iter()
            .map(Deref::deref)
            .map(KStrCPtr::from_str)
            .collect::<Vec<_>>();
        let start_ctx = ProcessStartContext {
            prg_resolution_base: self.resolution_base,
            prg_path: KStrCPtr::from_str(self.cmd.as_str()),
            environment: self.env,
            start_flags: self.flags.bits(),
            start_security_context: self.start_security_context,
            init_handles_len: self.init_handles.len() as c_ulong,
            init_handles: self.init_handles.as_ptr(),
            label: KStrCPtr::from_str(self.label.as_str()),
            proc_args_len: proc_args.len() as c_ulong,
            proc_args: proc_args.as_ptr(),
            init_namespace: self.namespace,
        };

        let mut hdl = MaybeUninit::uninit();

        crate::result::Error::from_code(unsafe { CreateProcess(&start_ctx, hdl.as_mut_ptr()) })?;

        Ok(CommandResult {
            hdl: unsafe { hdl.assume_init() },
        })
    }

    unsafe fn spawn_replace_image(&mut self) -> crate::result::Result<!> {
        self.spawn_with_result().map(|_| debug_unreachable())
    }
}

pub struct Stdio<'a>(
    HandlePtr<IOHandle>,
    PhantomData<BorrowedHandle<'a, IOHandle>>,
);

impl<'a> Stdio<'a> {
    pub const fn null() -> Self {
        Self(HandlePtr::null(), PhantomData)
    }
}

impl<'a, H: AsHandle<'a, IOHandle>> From<H> for Stdio<'a> {
    fn from(hdl: H) -> Stdio<'a> {
        Self(hdl.as_handle(), PhantomData)
    }
}

impl<'a> Command<'a> {
    pub fn new<P: AsRef<Path>>(cmd: P) -> Self {
        Self {
            resolution_base: HandlePtr::null(),
            cmd: cmd.as_ref().to_path_buf(),
            env: HandlePtr::null(),
            namespace: HandlePtr::null(),
            start_security_context: HandlePtr::null(),
            args: alloc::vec![cmd.as_ref().to_string()],
            init_handles: alloc::vec![
                unsafe { __HANDLE_IO_STDIN }.cast(),
                unsafe { __HANDLE_IO_STDOUT }.cast(),
                unsafe { __HANDLE_IO_STDERR }.cast()
            ],
            label: String::new(),
            flags: ProcessStartFlags::empty(),
            _handles: PhantomData,
        }
    }

    pub fn new_in<P: AsRef<Path>, H: AsHandle<'a, FileHandle>>(resolution_base: H, cmd: P) -> Self {
        Self {
            resolution_base: resolution_base.as_handle(),
            cmd: cmd.as_ref().to_path_buf(),
            env: HandlePtr::null(),
            namespace: HandlePtr::null(),
            start_security_context: HandlePtr::null(),
            args: alloc::vec![cmd.as_ref().to_string()],
            init_handles: alloc::vec![
                unsafe { __HANDLE_IO_STDIN }.cast(),
                unsafe { __HANDLE_IO_STDOUT }.cast(),
                unsafe { __HANDLE_IO_STDERR }.cast()
            ],
            label: String::new(),
            flags: ProcessStartFlags::empty(),
            _handles: PhantomData,
        }
    }

    pub fn init_handle<H, P: AsHandle<'a, H>>(&mut self, hdl: P) -> &mut Self {
        self.init_handles.push(hdl.as_handle().cast());
        self
    }

    pub fn stdin<P: AsHandle<'a, IOHandle>>(&mut self, hdl: P) -> &mut Self {
        self.init_handles[0] = hdl.as_handle().cast();
        self
    }

    pub fn stdout<P: AsHandle<'a, IOHandle>>(&mut self, hdl: P) -> &mut Self {
        self.init_handles[1] = hdl.as_handle().cast();
        self
    }

    pub fn stderr<P: AsHandle<'a, IOHandle>>(&mut self, hdl: P) -> &mut Self {
        self.init_handles[2] = hdl.as_handle().cast();
        self
    }
}

pub struct ProcessIterator {
    hdl: HandlePtr<EnumerateProcessHandle>,
    state: *mut c_void,
}
