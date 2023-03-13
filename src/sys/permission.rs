use core::ffi::c_long;

use super::{handle::{Handle,HandlePtr}, result::SysResult, kstr::KStrCPtr, thread::ThreadHandle, process::ProcessHandle};


#[repr(transparent)]
pub struct SecurityContext(Handle);

#[allow(improper_ctypes)]
extern "C"{
    pub fn CreateSecurityContext(nctx: *mut HandlePtr<SecurityContext>) -> SysResult;
    pub fn CopySecurityContext(nctx: *mut HandlePtr<SecurityContext>, ctx: HandlePtr<SecurityContext>) -> SysResult;
    pub fn DestroySecurityContext(ctx: HandlePtr<SecurityContext>) -> SysResult;
    pub fn GetCurrentSecurityContext(nctx: *mut HandlePtr<SecurityContext>) -> SysResult;
    pub fn HasKernelPermission(ctx: HandlePtr<SecurityContext>, perm: KStrCPtr) -> SysResult;
    pub fn HasThreadPermission(ctx: HandlePtr<SecurityContext>, th: HandlePtr<ThreadHandle>, perm: KStrCPtr) -> SysResult;
    pub fn HasProcessPermission(ctx: HandlePtr<SecurityContext>, ph: HandlePtr<ProcessHandle>, perm: KStrCPtr) -> SysResult;
    pub fn GrantKernelPermission(ctx: HandlePtr<SecurityContext>, perm: KStrCPtr, status: c_long) -> SysResult;
    pub fn GrantThreadPermission(ctx: HandlePtr<SecurityContext>, th: HandlePtr<ThreadHandle>, perm: KStrCPtr, status: c_long) -> SysResult;
    pub fn GrantProcessPermission(ctx: HandlePtr<SecurityContext>, ph: HandlePtr<ProcessHandle>, perm: KStrCPtr, status: c_long) -> SysResult;
    pub fn DropKernelPermission(ctx: HandlePtr<SecurityContext>, perm: KStrCPtr, status: c_long) -> SysResult;
    pub fn DropThreadPermission(ctx: HandlePtr<SecurityContext>, th: HandlePtr<ThreadHandle>, perm: KStrCPtr, status: c_long) -> SysResult;
    pub fn DropProcessPermission(ctx: HandlePtr<SecurityContext>, ph: HandlePtr<ProcessHandle>, perm: KStrCPtr, status: c_long) -> SysResult;
    pub fn RevokeKernelPermission(ctx: HandlePtr<SecurityContext>, perm: KStrCPtr) -> SysResult;
    pub fn RevokeThreadPermission(ctx: HandlePtr<SecurityContext>, th: HandlePtr<ThreadHandle>, perm: KStrCPtr) -> SysResult;
    pub fn RevokeProcessPermission(ctx: HandlePtr<SecurityContext>, ph: HandlePtr<ProcessHandle>, perm: KStrCPtr) -> SysResult;
}

