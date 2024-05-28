use crate::sys::kstr::KStrPtr;
use crate::sys::option::ExtendedOptionHead;
use crate::uuid::Uuid;

#[repr(C, align(32))]
pub struct ProcInfoRequestCleverCpuex {
    /// The header of the option.
    pub head: ExtendedOptionHead,
    /// Contains, in order, the values of cpuex2 through cpuex6.
    pub cpuex: [u64; 5],
}

#[repr(C, align(32))]
pub struct ProcInfoRequestCleverCpuid {
    /// The header of the option.
    pub head: ExtendedOptionHead,
    /// Contains the `cpuid` registers concatenated as a uuid
    pub cpuid: Uuid,
    /// If known, contains the Publically set machine name corresponding to the `cpuid` of the machine, otherwise set to an empty string.
    /// Strings are typically taken from the `cpuid-name` field in the official machine name registry <https://github.com/Clever-ISA/cpuid-names/blob/main/cpuid-names.csv>,
    ///  but they may be taken from other sources.
    pub cpu_machine_name: KStrPtr,
    /// If known, contains the Publically set vendor name corresponding to the `cpuid`, otherwise set to an empty string.
    /// Strings are typically taken from the `vendor` field in the official machine name registery <https://github.com/Clever-ISA/cpuid-names/blob/main/cpuid-names.csv>,
    ///  but they may be taken from other sources.
    pub cpu_vendor_name: KStrPtr,
}
