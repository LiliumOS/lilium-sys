use core::{
    mem::{ManuallyDrop, MaybeUninit},
    ptr::NonNull,
};

#[cfg(feature = "bytemuck")]
use bytemuck::{AnyBitPattern, NoUninit, Zeroable};

/// A type that can contain any value of type `T` or any initialized bitpattern.
/// More specifically, [`MaybeValid<T>`] is like [`MaybeUninit<T>`] but disallows uninitialized bytes anywhere other than padding (or other places uninit is allowed, such as inside an internal [`MaybeUninit<T>`])
///
/// Like [`MaybeUninit<T>`], [`MaybeValid<T>`] has the same size, alignment, and abi as `T`. Note that this include padding bytes, which are lost (reset to uninit)
///
/// ## Formal Safety
/// This section explains a formalized definition of the invariant of [`MaybeValid<T>`].
///
/// Let `b` be a byte offset into some type `U`. Such an offset `b` is *uninit-allowing* only if:
/// * `U` is [`MaybeUninit<R>`] or a `MaybeUninit`-similar type for some type `R`,
/// * If it a padding byte inside `T` or any field therein, or
/// * `b` is at offset `o` within a field `f` of some type `R`, and offset `o` is *uninit-allowing* in R.
///
/// A value of [`MaybeValid<T>`] is valid if it is computed from a sequence of bytes `r` with length `T`, such that for each byte `b` in `r`:
/// * The `b`` has an initialized value, or
/// * The byte offset in `T` corresponding to `b` is *uninit-allowing*`
///
/// The requirements of *uninit-allowing* depends on the public interface of the type:
/// The general rule is, if a safe value of the type can be constructed with an uninit byte value at some offset, it's correct for a [`MaybeValid<T>`] to contain an uninit byte at that offset.
///
/// If the type disallows uninit bytes everywhere (IE. a scalar value), then so does [`MaybeValid<T>`].
///
/// ## Implementation Note
/// The current implementation of [`MaybeValid<T>`] contains a [`MaybeUninit<T>`]. However, this does not allow arbitrary uninit bytes within the value.
/// Instead [`MaybeValid<T>`] is transparent in the *uninit-allowing* predicate above: A byte `b` in [`MaybeValid<T>`] is *uninit-allowing*
#[repr(transparent)]
pub struct MaybeValid<T>(MaybeUninit<T>);

impl<T> MaybeValid<T> {
    /// Constructs a [`MaybeValid`] that contains a given `T`
    pub const fn new(val: T) -> Self {
        Self(MaybeUninit::new(val))
    }

    /// Constructs a [`MaybeValid<T>`] that contains all zero bytes
    pub const fn zeroed() -> Self {
        Self(MaybeUninit::zeroed())
    }

    /// Constructs a [`MaybeValid<T>`] that contains `n` repeated infinitely.
    ///
    /// Note that this has the same caveats as [`MaybeValid::zeroed`], and padding bytes in `T` are discarded
    pub const fn fill(n: u8) -> Self {
        let mut uninit = MaybeUninit::<T>::uninit();

        let ptr = uninit.as_mut_ptr().cast::<u8>();

        let mut i = core::mem::size_of::<T>();

        while i > 0 {
            i -= 1;
            unsafe { ptr.add(i).write(n) }
        }

        // SAFETY: We initialized every non-padding byte in `T` to an initialized value
        Self(uninit)
    }

    /// Constructs a [`MaybeValid<T>`] by assuming that [`MaybeUninit<T>`] contains an initialized (but not necessarily valid) value.
    ///
    /// This is slightly more permissive than calling [`MaybeValid::new`] because it does not require `x` to be valid.
    pub const unsafe fn from_uninit_unchecked(x: MaybeUninit<T>) -> Self {
        Self(x)
    }

    /// Assumes that the contained value is a valid (and safe) value of type `T`.
    ///
    /// # Safety
    /// The [`MaybeValid<T>`] must contain a value of type `T` that satisfies the invariants of `T`.
    pub const unsafe fn assume_valid(self) -> T {
        unsafe { self.0.assume_init() }
    }

    /// Converts a [`MaybeValid<T>`] to a [`MaybeUninit<T>`]
    pub const fn into_uninit(self) -> MaybeUninit<T> {
        self.0
    }
}

#[cfg(feature = "bytemuck")]
impl<T: AnyBitPattern> MaybeValid<T> {
    /// Converts safely from [`MaybeValid`] to the inner type, with [`bytemuck`] proving that the operation is safe.
    ///
    /// This is the same as [`bytemuck::cast`], except it's a `const fn`, is statically known not to panic, and is allowed when `T` is not [`NoUninit`],
    ///  which is safe because [`MaybeValid<T>`] can't carry uninit bytes outside of inner padding bytes or [`MaybeUninit`] fields within `T`.
    pub const fn assert_valid(self) -> T {
        // SAFETY: Asserted by `AnyBitPattern` bound
        unsafe { self.assume_valid() }
    }
}

impl<T: Copy> Copy for MaybeValid<T> {}
impl<T: Copy> Clone for MaybeValid<T> {
    fn clone(&self) -> Self {
        *self
    }
}

#[cfg(feature = "bytemuck")]
unsafe impl<T: Copy + 'static> AnyBitPattern for MaybeValid<T> {}
#[cfg(feature = "bytemuck")]
unsafe impl<T: NoUninit> NoUninit for MaybeValid<T> {}
#[cfg(feature = "bytemuck")]
unsafe impl<T> Zeroable for MaybeValid<T> {}

macro_rules! impl_debug_int {
    ($($ty:ty),*) => {
        $(
            impl core::fmt::Debug for MaybeValid<$ty> {
                fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                    // SAFETY:
                    // `$ty` is a primitive type and all uninit values are valid
                    let v = unsafe { self.assume_valid() };
                    v.fmt(f)
                }
            }
            impl core::fmt::Debug for MaybeValid<core::num::NonZero<$ty>> {
                fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                    // SAFETY:
                    // `$ty` is a primitive type and all uninit values are valid
                    let v: $ty = unsafe { transmute_checked(*self) };
                    v.fmt(f)
                }
            }

            impl core::fmt::Debug for MaybeValid<Option<core::num::NonZero<$ty>>> {
                fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                    // SAFETY:
                    // `$ty` is a primitive type and all uninit values are valid
                    let v = unsafe { self.assume_valid() };
                    v.fmt(f)
                }
            }
            impl core::fmt::UpperHex for MaybeValid<$ty> {
                fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                    // SAFETY:
                    // `$ty` is a primitive type and all uninit values are valid
                    let v = unsafe { self.assume_valid() };
                    v.fmt(f)
                }
            }
            impl core::fmt::UpperHex for MaybeValid<core::num::NonZero<$ty>> {
                fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                    // SAFETY:
                    // `$ty` is a primitive type and all uninit values are valid
                    let v: $ty = unsafe { transmute_checked(*self) };
                    v.fmt(f)
                }
            }

            impl core::fmt::LowerHex for MaybeValid<$ty> {
                fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                    // SAFETY:
                    // `$ty` is a primitive type and all uninit values are valid
                    let v = unsafe { self.assume_valid() };
                    v.fmt(f)
                }
            }
            impl core::fmt::LowerHex for MaybeValid<core::num::NonZero<$ty>> {
                fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                    // SAFETY:
                    // `$ty` is a primitive type and all uninit values are valid
                    let v: $ty = unsafe { transmute_checked(*self) };
                    v.fmt(f)
                }
            }

            impl core::fmt::Octal for MaybeValid<$ty> {
                fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                    // SAFETY:
                    // `$ty` is a primitive type and all uninit values are valid
                    let v = unsafe { self.assume_valid() };
                    v.fmt(f)
                }
            }
            impl core::fmt::Octal for MaybeValid<core::num::NonZero<$ty>> {
                fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                    // SAFETY:
                    // `$ty` is a primitive type and all uninit values are valid
                    let v: $ty = unsafe { transmute_checked(*self) };
                    v.fmt(f)
                }
            }

            impl core::fmt::Binary for MaybeValid<$ty> {
                fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                    // SAFETY:
                    // `$ty` is a primitive type and all uninit values are valid
                    let v = unsafe { self.assume_valid() };
                    v.fmt(f)
                }
            }
            impl core::fmt::Binary for MaybeValid<core::num::NonZero<$ty>> {
                fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                    // SAFETY:
                    // `$ty` is a primitive type and all uninit values are valid
                    let v: $ty = unsafe { transmute_checked(*self) };
                    v.fmt(f)
                }
            }
        )*
    }
}

impl_debug_int!(i8, u8, i16, u16, i32, u32, i64, u64, i128, u128, isize, usize);

impl<T> core::fmt::Debug for MaybeValid<*const T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let v = unsafe { self.assume_valid() };
        v.fmt(f)
    }
}

impl<T> core::fmt::Debug for MaybeValid<*mut T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let v = unsafe { self.assume_valid() };
        v.fmt(f)
    }
}

impl<T> core::fmt::Debug for MaybeValid<NonNull<T>> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let v: *const T = unsafe { transmute_checked(*self) };

        v.fmt(f)
    }
}

impl<T> core::fmt::Debug for MaybeValid<Option<NonNull<T>>> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let v = unsafe { self.assume_valid() };
        v.fmt(f)
    }
}

impl<T> core::fmt::Pointer for MaybeValid<*const T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let v = unsafe { self.assume_valid() };
        v.fmt(f)
    }
}

impl<T> core::fmt::Pointer for MaybeValid<*mut T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let v = unsafe { self.assume_valid() };
        v.fmt(f)
    }
}

impl<T> core::fmt::Pointer for MaybeValid<NonNull<T>> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let v: *const T = unsafe { transmute_checked(*self) };

        v.fmt(f)
    }
}

impl<'a, T> core::fmt::Pointer for MaybeValid<&'a T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let v: *const T = unsafe { transmute_checked(*self) };

        v.fmt(f)
    }
}

impl<'a, T> core::fmt::Pointer for MaybeValid<&'a mut T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let v: *const T = unsafe { core::ptr::read((self as *const Self).cast::<*const T>()) };

        v.fmt(f)
    }
}

macro_rules! impl_fn_fmt_sig {
    (for<$($param:ident),*> $fn_ty:ty) => {
        impl<R, $($param),*> core::fmt::Debug for MaybeValid<$fn_ty> {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                let v: *const () = unsafe{transmute_checked(*self)};

                v.fmt(f)
            }
        }
        impl<R, $($param),*> core::fmt::Pointer for MaybeValid<$fn_ty> {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                let v: *const () = unsafe{transmute_checked(*self)};

                v.fmt(f)
            }
        }
        impl<R, $($param),*> core::fmt::Debug for MaybeValid<Option<$fn_ty>> {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                let v = unsafe{self.assume_valid()};

                v.fmt(f)
            }
        }
    }
}

macro_rules! impl_fn_fmt {
    ($($param:ident),*) => {
        impl_fn_fmt_sig!(for<$($param),*> fn($($param),*)->R);
        impl_fn_fmt_sig!(for<$($param),*> unsafe fn($($param),*)->R);
        impl_fn_fmt_sig!(for<$($param),*> extern "C" fn($($param),*)->R);
        impl_fn_fmt_sig!(for<$($param),*> unsafe extern "C" fn($($param),*)->R);
        impl_fn_fmt_sig!(for<$($param),*> extern "system" fn($($param),*)->R);
        impl_fn_fmt_sig!(for<$($param),*> unsafe extern "system" fn($($param),*)->R);
        impl_fn_fmt_sig!(for<$($param),*> extern "C-unwind" fn($($param),*)->R);
        impl_fn_fmt_sig!(for<$($param),*> unsafe extern "C-unwind" fn($($param),*)->R);
        impl_fn_fmt_sig!(for<$($param),*> extern "system-unwind" fn($($param),*)->R);
        impl_fn_fmt_sig!(for<$($param),*> unsafe extern "system-unwind" fn($($param),*)->R);
    };
}

impl_fn_fmt!();
impl_fn_fmt!(A);
impl_fn_fmt!(A, B);
impl_fn_fmt!(A, B, C);
impl_fn_fmt!(A, B, C, D);
impl_fn_fmt!(A, B, C, D, E);
impl_fn_fmt!(A, B, C, D, E, F);
impl_fn_fmt!(A, B, C, D, E, F, G);
impl_fn_fmt!(A, B, C, D, E, F, G, H);
impl_fn_fmt!(A, B, C, D, E, F, G, H, I);
impl_fn_fmt!(A, B, C, D, E, F, G, H, I, J);
impl_fn_fmt!(A, B, C, D, E, F, G, H, I, J, K);
impl_fn_fmt!(A, B, C, D, E, F, G, H, I, J, K, L);

/// Performs the same operation as [`transmute`][core::mem::transmute], but ignoring the size check.
///
/// # Safety
/// It must be valid to [`transmute`][core::mem::transmute] from `x` to `U`.
/// Additionally, `T` and `U` must be the same size.
pub const unsafe fn transmute_unchecked<T, U>(x: T) -> U {
    core::hint::assert_unchecked(const { core::mem::size_of::<T>() == core::mem::size_of::<U>() });
    union Transmuter<T, U> {
        x: ManuallyDrop<T>,
        y: ManuallyDrop<U>,
    }

    ManuallyDrop::into_inner(
        Transmuter {
            x: ManuallyDrop::new(x),
        }
        .y,
    )
}

/// Performs the same operation as [`transmute`][core::mem::transmute], but performs the size check on monomorphized types.
/// This allows the function to be used in a generic context where sizes might statically differ.
///
/// # Safety
/// Has the same preconditions as [`transmute`][core::mem::transmute].
pub const unsafe fn transmute_checked<T, U>(x: T) -> U {
    const {
        assert!(core::mem::size_of::<T>() == core::mem::size_of::<U>());
    }

    // SAFETY: We've statically checked the only additional precondition that `transmute_unchecked` adds.
    unsafe { transmute_unchecked(x) }
}
