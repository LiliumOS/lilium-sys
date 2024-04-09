use core::ffi::{c_long, c_ulong};

use crate::uuid::Uuid;

use super::{
    handle::{Handle, HandlePtr, WideHandle},
    kstr::KStrCPtr,
    process::ProcessHandle,
    result::SysResult,
    thread::ThreadHandle,
};

#[repr(transparent)]
pub struct SecurityContext(Handle);

#[repr(C)]
pub union ThreadOwner {
    pub process: WideHandle<ProcessHandle>,
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

    /// Sets the primary principal of the security context.
    ///
    /// ## Permission Check
    ///
    /// ctx must have principal as one of it's principals (either an explicit primary or an explicit secondary principal), or the thread must have the kernel permission SECURITY_SET_CREDENTIAL.
    pub fn SetPrimaryPrincipal(ctx: HandlePtr<SecurityContext>, principal: *const Uuid);

    pub fn AddSecondaryPrincipal(ctx: HandlePtr<SecurityContext>, principal: *const Uuid);

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

    /// Gets the value of limit on a kernel (global) resource in the given security context. Note that this is the maximum value for the limit, not the current in-use value.
    /// The Debug API can provide some of this information.
    ///
    /// ### Known resource limits
    ///
    /// The set of resource Limits supported is kernel dependant.
    /// The following resource limit names are defined:
    /// * `Memory`: Total virtual memory allocated for process/thread mappings
    /// * `PhysMem`: Total physical memory allocated for process/thread mappings
    /// * `KMemory`: Total virtual memory allocated
    pub fn GetKernelResourceLimit(
        ctx: HandlePtr<SecurityContext>,
        limit_name: KStrCPtr,
        value: *mut u64,
    ) -> SysResult;

    pub fn EncodeSecurityContext(
        ctx: HandlePtr<SecurityContext>,
        buffer: *mut u8,
        len: *mut usize,
    ) -> SysResult;

    pub fn GetPrimaryPrincipal(ctx: HandlePtr<SecurityContext>, principal: *mut Uuid) -> SysResult;
    pub fn GetSecondaryPrincipals(
        ctx: HandlePtr<SecurityContext>,
        principals: *mut Uuid,
        principals_len: *mut c_ulong,
    ) -> SysResult;
}
