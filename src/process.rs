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

use crate::sys::except::{ExceptionInfo, ExceptionStatusInfo};
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
            self as sys, CreateProcess, EnumerateProcessHandle, EnvironmentMapHandle, ProcessHandle,
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

struct CommandResult {
    hdl: HandlePtr<ProcessHandle>,
}

impl CommandResult {
    fn join(self) -> crate::result::Result<CommandStatus> {
        let mut sigterminfo = MaybeUninit::zeroed();
        loop {
            let ret =
                unsafe { crate::sys::process::JoinProcess(self.hdl, sigterminfo.as_mut_ptr()) };
            match crate::result::Error::from_code(ret) {
                Ok(()) => break Ok(CommandStatus::Normal(ret as i32)), // Note: Lilium guarantees it will be a positive i32
                Err(crate::result::Error::Signaled) => {
                    break Ok(CommandStatus::UnmanagedException(unsafe {
                        sigterminfo.assume_init()
                    }))
                }
                Err(crate::result::Error::Killed) => break Ok(CommandStatus::Killed),
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
        todo!()
    }

    unsafe fn spawn_replace_image(&mut self) -> crate::result::Result<!> {
        unsafe {
            self.flags |= ProcessStartFlags::REPLACE_IMAGE;
            self.spawn_with_result().map(|_| debug_unreachable())
        }
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

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum CommandStatus {
    Normal(i32),
    UnmanagedException(ExceptionStatusInfo),
    Killed,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct ExitStatus(CommandStatus);

impl ExitStatus {
    pub fn exit_ok(&self) -> core::result::Result<(), ExitStatusError> {
        if matches!(self.0, CommandStatus::Normal(0)) {
            Ok(())
        } else {
            Err(ExitStatusError(self.0))
        }
    }

    pub fn success(&self) -> bool {
        matches!(self.0, CommandStatus::Normal(0))
    }
}

impl ExitStatus {
    pub fn throw_except(&self) -> crate::result::Result<()> {
        if let CommandStatus::UnmanagedException(except) = &self.0 {
            crate::result::Error::from_code(unsafe {
                crate::sys::except::ExceptHandleSynchronous(except, core::ptr::null())
            })
        } else {
            Ok(())
        }
    }

    pub fn exit_code(&self) -> Option<i32> {
        if let &CommandStatus::Normal(status) = &self.0 {
            Some(status)
        } else {
            None
        }
    }

    pub fn exception(&self) -> Option<&ExceptionStatusInfo> {
        if let CommandStatus::UnmanagedException(except) = &self.0 {
            Some(except)
        } else {
            None
        }
    }

    pub fn killed(&self) -> bool {
        matches!(self.0, CommandStatus::Killed)
    }

    pub fn abnormal(&self) -> bool {
        matches!(
            self.0,
            CommandStatus::Killed | CommandStatus::UnmanagedException(_)
        )
    }
}

impl Default for ExitStatus {
    fn default() -> Self {
        Self(CommandStatus::Normal(0))
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct ExitStatusError(CommandStatus);

impl ExitStatusError {
    pub fn throw_except(&self) -> crate::result::Result<()> {
        if let CommandStatus::UnmanagedException(except) = &self.0 {
            crate::result::Error::from_code(unsafe {
                crate::sys::except::ExceptHandleSynchronous(except, core::ptr::null())
            })
        } else {
            Ok(())
        }
    }

    pub fn exit_code(&self) -> Option<i32> {
        if let &CommandStatus::Normal(status) = &self.0 {
            Some(status)
        } else {
            None
        }
    }

    pub fn exception(&self) -> Option<&ExceptionStatusInfo> {
        if let CommandStatus::UnmanagedException(except) = &self.0 {
            Some(except)
        } else {
            None
        }
    }

    pub fn killed(&self) -> bool {
        matches!(self.0, CommandStatus::Killed)
    }

    pub fn abnormal(&self) -> bool {
        matches!(
            self.0,
            CommandStatus::Killed | CommandStatus::UnmanagedException(_)
        )
    }
}

impl From<ExitStatusError> for ExitStatus {
    fn from(value: ExitStatusError) -> Self {
        ExitStatus(value.0)
    }
}
