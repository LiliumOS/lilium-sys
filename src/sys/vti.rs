use core::ffi::c_void;

pub mod arch {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    pub mod x86;
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    pub use x86::*;
}

#[repr(C)]
pub union FaultInfo {
    pub arch: arch::ArchFaultInfo,
    pub generic: [*mut c_void; 32],
}

#[repr(C)]
pub struct VirtualizationCallbacks {
    pub fault_info_buf: *mut FaultInfo,
    pub callback_udata: *mut c_void,
    pub callback_page_alloc: Option<unsafe extern "C" fn(*mut c_void, u64) -> *mut c_void>,
    pub callback_fault:
        Option<unsafe extern "C" fn(*mut c_void, u32, fault_info: *mut FaultInfo) -> u32>,
}
