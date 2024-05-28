use core::ffi::c_void;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ArchFaultInfo {
    pub fault_code: usize,
    pub pfla: *mut c_void,
    pub save_ss: usize,
    pub save_cs: usize,
    pub save_ip: *mut c_void,
    pub save_sp: *mut c_void,
}

#[repr(C)]
pub struct Registers {
    pub ax: usize,
    pub cx: usize,
    pub dx: usize,
    pub bx: usize,
    pub sp: usize,
    pub bp: usize,
    pub si: usize,
    pub di: usize,
    #[cfg(target_arch = "x86_64")]
    pub rx: [usize; 8],
    pub ss: u16,
    pub cs: u16,
    pub ds: u16,
    pub es: u16,
    pub fs: u16,
    pub gs: u16,
    pub resseg: [u16; 2],
    pub flags: usize,
    pub ip: usize,
    pub cr0: usize,
    pub cr1: usize,
    pub cr2: usize,
    pub cr3: usize,
    pub cr4: usize,
    pub rescr: [usize; 3],
    #[cfg(target_arch = "x86_64")]
    pub cr8: usize,
    #[cfg(target_arch = "x86_64")]
    pub rescrn: [usize; 7],
    #[cfg(target_arch = "x86_64")]
    pub rx_apx: [usize; 8],
}
