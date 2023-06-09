use core::ffi::c_long;

use crate::uuid::Uuid;

use super::{
    handle::{Handle, HandlePtr},
    kstr::KStrCPtr,
    process::ProcessHandle,
    result::SysResult,
    thread::ThreadHandle,
};

#[repr(transparent)]
pub struct SecurityContext(Handle);

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ThreadOwnerProcess {
    pub handle: HandlePtr<ProcessHandle>,
    #[doc(hidden)]
    pub __padding: [u32; (16 - core::mem::size_of::<HandlePtr<ProcessHandle>>()) >> 2],
}

#[repr(C)]
pub union ThreadOwner {
    pub process: ThreadOwnerProcess,
    pub owning_principal: Uuid,
}

#[allow(improper_ctypes)]
extern "C" {
    pub fn CreateSecurityContext(nctx: *mut HandlePtr<SecurityContext>) -> SysResult;
    pub fn CopySecurityContext(
        nctx: *mut HandlePtr<SecurityContext>,
        ctx: HandlePtr<SecurityContext>,
    ) -> SysResult;
    pub fn DestroySecurityContext(ctx: HandlePtr<SecurityContext>) -> SysResult;
    pub fn GetCurrentSecurityContext(nctx: *mut HandlePtr<SecurityContext>) -> SysResult;
    pub fn HasKernelPermission(ctx: HandlePtr<SecurityContext>, perm: KStrCPtr) -> SysResult;
    pub fn HasThreadPermission(
        ctx: HandlePtr<SecurityContext>,
        th: HandlePtr<ThreadHandle>,
        perm: KStrCPtr,
    ) -> SysResult;
    pub fn HasProcessPermission(
        ctx: HandlePtr<SecurityContext>,
        ph: HandlePtr<ProcessHandle>,
        perm: KStrCPtr,
    ) -> SysResult;
    pub fn GrantKernelPermission(
        ctx: HandlePtr<SecurityContext>,
        perm: KStrCPtr,
        status: c_long,
    ) -> SysResult;
    pub fn GrantThreadPermission(
        ctx: HandlePtr<SecurityContext>,
        th: HandlePtr<ThreadHandle>,
        perm: KStrCPtr,
        status: c_long,
    ) -> SysResult;
    pub fn GrantProcessPermission(
        ctx: HandlePtr<SecurityContext>,
        ph: HandlePtr<ProcessHandle>,
        perm: KStrCPtr,
        status: c_long,
    ) -> SysResult;
    pub fn DropKernelPermission(
        ctx: HandlePtr<SecurityContext>,
        perm: KStrCPtr,
        status: c_long,
    ) -> SysResult;
    pub fn DropThreadPermission(
        ctx: HandlePtr<SecurityContext>,
        th: HandlePtr<ThreadHandle>,
        perm: KStrCPtr,
        status: c_long,
    ) -> SysResult;
    pub fn DropProcessPermission(
        ctx: HandlePtr<SecurityContext>,
        ph: HandlePtr<ProcessHandle>,
        perm: KStrCPtr,
        status: c_long,
    ) -> SysResult;
    pub fn RevokeKernelPermission(ctx: HandlePtr<SecurityContext>, perm: KStrCPtr) -> SysResult;
    pub fn RevokeThreadPermission(
        ctx: HandlePtr<SecurityContext>,
        th: HandlePtr<ThreadHandle>,
        perm: KStrCPtr,
    ) -> SysResult;
    pub fn RevokeProcessPermission(
        ctx: HandlePtr<SecurityContext>,
        ph: HandlePtr<ProcessHandle>,
        perm: KStrCPtr,
    ) -> SysResult;

    pub fn SetKernelResourceLimit(
        ctx: HandlePtr<SecurityContext>,
        limit_name: KStrCPtr,
        value: u64,
    ) -> SysResult;


    pub fn EncodeSecurityContext(ctx: HandlePtr<SecurityContext>, buffer: *mut u8, len: *mut usize) -> SysResult;
}
