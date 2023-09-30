use core::mem::MaybeUninit;

use crate::sys::handle::HandlePtr;
use crate::sys::kstr::KStrCPtr;
pub use crate::sys::permission::SecurityContext;

use crate::sys::process::ProcessHandle;
use crate::sys::thread::ThreadHandle;
use crate::{handle::*, result::Error, sys::permission::*};

bitflags::bitflags! {
    #[repr(transparent)]
    #[derive(Copy, Clone, Hash, PartialEq, Debug)]
    pub struct PermissionStatus : core::ffi::c_long{
        const ALLOWED = 0x01;
        const INHERITABLE = 0x02;
        const RECOVERABLE = 0x04;
        const REVOKED = 0x08;
    }
}

pub fn has_kernel_permission(perm: &str) -> crate::result::Result<PermissionStatus> {
    let status = unsafe { HasKernelPermission(HandlePtr::null(), KStrCPtr::from_str(perm)) };
    Error::from_code(status)?;
    Ok(PermissionStatus::from_bits_retain(status))
}

pub fn has_thread_permission(
    th: &HandleRef<ThreadHandle>,
    perm: &str,
) -> crate::result::Result<PermissionStatus> {
    let status =
        unsafe { HasThreadPermission(HandlePtr::null(), th.as_raw(), KStrCPtr::from_str(perm)) };
    Error::from_code(status)?;
    Ok(PermissionStatus::from_bits_retain(status))
}

pub fn has_process_permission(
    ph: &HandleRef<ProcessHandle>,
    perm: &str,
) -> crate::result::Result<PermissionStatus> {
    let status =
        unsafe { HasProcessPermission(HandlePtr::null(), ph.as_raw(), KStrCPtr::from_str(perm)) };
    Error::from_code(status)?;
    Ok(PermissionStatus::from_bits_retain(status))
}

impl SecurityContext {
    pub fn new() -> crate::result::Result<OwnedHandle<Self>> {
        let mut ctx = MaybeUninit::zeroed();
        Error::from_code(unsafe { CreateSecurityContext(ctx.as_mut_ptr()) })?;
        Ok(unsafe { OwnedHandle::take_ownership(ctx.assume_init()) })
    }

    pub fn current() -> crate::result::Result<OwnedHandle<Self>> {
        let mut ctx = MaybeUninit::zeroed();
        Error::from_code(unsafe { GetCurrentSecurityContext(ctx.as_mut_ptr()) })?;
        Ok(unsafe { OwnedHandle::take_ownership(ctx.assume_init()) })
    }
}

impl HandleRef<SecurityContext> {
    pub fn clone(&self) -> crate::result::Result<OwnedHandle<SecurityContext>> {
        let mut ctx = MaybeUninit::zeroed();
        Error::from_code(unsafe { CopySecurityContext(ctx.as_mut_ptr(), self.as_raw()) })?;
        Ok(unsafe { OwnedHandle::take_ownership(ctx.assume_init()) })
    }

    pub fn has_kernel_permission(&self, perm: &str) -> crate::result::Result<PermissionStatus> {
        let status = unsafe { HasKernelPermission(self.as_raw(), KStrCPtr::from_str(perm)) };
        Error::from_code(status)?;
        Ok(PermissionStatus::from_bits_retain(status))
    }

    pub fn has_thread_permission(
        &self,
        th: &HandleRef<ThreadHandle>,
        perm: &str,
    ) -> crate::result::Result<PermissionStatus> {
        let status =
            unsafe { HasThreadPermission(self.as_raw(), th.as_raw(), KStrCPtr::from_str(perm)) };
        Error::from_code(status)?;
        Ok(PermissionStatus::from_bits_retain(status))
    }

    pub fn has_process_permission(
        &self,
        th: &HandleRef<ProcessHandle>,
        perm: &str,
    ) -> crate::result::Result<PermissionStatus> {
        let status =
            unsafe { HasProcessPermission(self.as_raw(), th.as_raw(), KStrCPtr::from_str(perm)) };
        Error::from_code(status)?;
        Ok(PermissionStatus::from_bits_retain(status))
    }

    
}
