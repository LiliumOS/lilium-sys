use core::{arch::x86_64::__m128, ffi::c_void};

use crate::sys::kstr::KCSlice;

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
    pub xcr0: usize,
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
    pub dr: [usize; 8],
}

#[repr(C)]
#[cfg_attr(target_arch = "x86_64", repr(align(16)))]
#[derive(Copy, Clone)]
#[cfg_attr(feature = "bytemuck", derive(bytemuck::Zeroable))]
pub struct Fp80 {
    pub mant: u64,
    pub exp_and_sign: u16,
    #[doc(hidden)]
    pub __reserved: [u16; 3],
}

#[cfg(target_arch = "x86_64")]
pub const MAX_RNUM: usize = 16;
#[cfg(target_arch = "x86")]
pub const MAX_RNUM: usize = 8;

#[repr(C, align(512))]
#[derive(Copy, Clone)]
#[cfg_attr(feature = "bytemuck", derive(bytemuck::Zeroable))]
pub struct FxsaveState {
    pub fcw: u16,
    pub fsw: u16,
    pub ftw: u8,
    #[doc(hidden)]
    pub __reserved5: u8,
    pub fop: u16,
    pub fip: usize,
    #[cfg(target_arch = "x86")]
    pub fcs: u16,
    #[cfg(target_arch = "x86")]
    #[doc(hidden)]
    pub __reserved14: u16,
    pub st: [Fp80; 8],
    pub xmm: [__m128; MAX_RNUM],
    #[doc(hidden)]
    pub __reserved_rest: [u128; 19 - MAX_RNUM],
    /// Region that is not modified by the kernel
    pub avail_region: [u128; 3],
}

impl FxsaveState {
    pub const ZERO: Self = unsafe { core::mem::zeroed() };
}

#[repr(C, align(512))]
#[derive(Copy, Clone)]
#[cfg_attr(feature = "bytemuck", derive(bytemuck::Zeroable))]
pub struct X86FpState {
    pub fxsave: FxsaveState,
}

def_option_type! {
    pub struct VmConfigOptionX86EnabledMsr("1f94620f-0abd-51e1-b0a8-76ecaa34385f"){
        exposed_msrs: KCSlice<u32>
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
#[cfg_attr(feature = "bytemuck", derive(bytemuck::Zeroable, bytemuck::Pod))]
pub struct CpuidLeaf {
    /// The leaf (input `eax` value) to respond to
    pub leaf: u32,
    /// The subleaf (input `ecx` value) to respond to.
    /// `0xFFFFFFFF` means ignoring the subleaf
    pub subleaf: u32,
    /// The output of the `cpuid` instruction, specified in register number order (eax, ecx, edx, ebx)
    pub output: [u32; 4],
}

def_option_type! {
    pub struct VmConfigOptionCpuid("1f94620f-0abd-51e1-b0a8-76ecaa34385f"){
        cpuid_id: KCSlice<CpuidLeaf>
    }
}

def_option! {
    pub union VmConfigOptionArch(64) {}
}
