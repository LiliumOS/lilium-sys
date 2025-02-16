//! The format of an encoding security context is a constant table followed by sequence of instructions.
//! Each instruction is exactly 16 bytes wide - a 32-bit opcode followed by 3 32-bit operands
//! The instruction format is very simple, and specifically is designed to match the goals of programs like the session-login-service, and the `InstallSecurityContext` stream.
//!
//!

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "bytemuck", derive(bytemuck::Zeroable, bytemuck::Pod))]
#[repr(transparent)]
pub struct ConstEnt(u32);

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
#[repr(u32)]
#[non_exhaustive]
pub enum ResetWhat {
    Nothing = 0,
    PrimaryPrincipal = 1,
    SecondaryPrincipals = 2,

    KernelPermissions = 4,
    ThreadPermissions = 5,
    ProcessPermissions = 6,
}

#[cfg(feature = "bytemuck")]
unsafe impl bytemuck::Zeroable for ResetWhat {}

#[cfg(feature = "bytemuck")]
unsafe impl bytemuck::NoUninit for ResetWhat {}

#[cfg(feature = "bytemuck")]
unsafe impl bytemuck::CheckedBitPattern for ResetWhat {
    type Bits = u32;

    fn is_valid_bit_pattern(bits: &Self::Bits) -> bool {
        match bits {
            0 | 1 | 2 | 4 | 5 | 6 => true,
            _ => false,
        }
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
#[repr(u32)]
#[non_exhaustive]
pub enum PermissionKind {
    Kernel = 0,
    Thread = 1,
    Process = 2,
}

#[cfg(feature = "bytemuck")]
unsafe impl bytemuck::Zeroable for PermissionKind {}

#[cfg(feature = "bytemuck")]
unsafe impl bytemuck::NoUninit for PermissionKind {}

#[cfg(feature = "bytemuck")]
unsafe impl bytemuck::CheckedBitPattern for PermissionKind {
    type Bits = u32;

    fn is_valid_bit_pattern(bits: &Self::Bits) -> bool {
        match bits {
            0 | 1 | 2 => true,
            _ => false,
        }
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
#[repr(u32)]
#[non_exhaustive]
pub enum SecurityContextInstruction {
    Nop([u32; 3]) = 0,
    Reset {
        what: ResetWhat,
        #[doc(hidden)]
        __rest: [u32; 2],
    } = 1,
    GrantPermission {
        kind: PermissionKind,
        perm: ConstEnt,
        ctx_sel: ConstEnt,
    } = 2,
    DropPermission {
        kind: PermissionKind,
        perm: ConstEnt,
        ctx_sel: ConstEnt,
    } = 3,
    RevekePermission {
        kind: PermissionKind,
        perm: ConstEnt,
        ctx_sel: ConstEnt,
    } = 4,
    SetPrimaryPrincipal {
        principal: ConstEnt,
        #[doc(hidden)]
        __rest: [u32; 2],
    } = 5,
    AddSecondaryPrincipal {
        principal: ConstEnt,
        #[doc(hidden)]
        __rest: [u32; 2],
    } = 6,
    DropSecondaryPrincipal {
        principal: ConstEnt,
        #[doc(hidden)]
        __rest: [u32; 2],
    } = 7,
}

#[cfg(feature = "bytemuck")]
unsafe impl bytemuck::Zeroable for SecurityContextInstruction {}

#[cfg(feature = "bytemuck")]
unsafe impl bytemuck::NoUninit for SecurityContextInstruction {}

#[cfg(feature = "bytemuck")]
unsafe impl bytemuck::CheckedBitPattern for SecurityContextInstruction {
    type Bits = [u32; 4];

    fn is_valid_bit_pattern(bits: &Self::Bits) -> bool {
        match bits {
            [0, _, _, _] => true,
            [1, what, 0, 0] => ResetWhat::is_valid_bit_pattern(bytemuck::must_cast_ref(what)),
            [2 | 3 | 4, kind, perm, ctx_sel] => {
                PermissionKind::is_valid_bit_pattern(bytemuck::must_cast_ref(kind))
                    && ConstEnt::is_valid_bit_pattern(bytemuck::must_cast_ref(perm))
                    && ConstEnt::is_valid_bit_pattern(bytemuck::must_cast_ref(ctx_sel))
            }
            [5 | 6 | 7, principal, 0, 0] => {
                ConstEnt::is_valid_bit_pattern(bytemuck::must_cast_ref(principal))
            }
            _ => false,
        }
    }
}
