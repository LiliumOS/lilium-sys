use crate::uuid::Uuid;

use super::{
    kstr::{KCSlice, KSlice, KStrCPtr, KStrPtr},
    option::ExtendedOptionHead,
};

pub trait EmptyVal {
    const EMPTY: Self;
}

macro_rules! def_empty_primitives {
    ($($ty:ident),*) => {
        $(
            impl EmptyVal for $ty {
                const EMPTY: Self = 0;
            }
        )*
    }
}

def_empty_primitives!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);

impl EmptyVal for Uuid {
    const EMPTY: Self = Uuid::NIL;
}

impl<T: EmptyVal, const N: usize> EmptyVal for [T; N] {
    const EMPTY: Self = [const { T::EMPTY }; N];
}

impl EmptyVal for KStrCPtr {
    const EMPTY: Self = KStrCPtr::empty();
}

impl EmptyVal for KStrPtr {
    const EMPTY: Self = KStrPtr::empty();
}

impl<T> EmptyVal for KCSlice<T> {
    const EMPTY: Self = KCSlice::empty();
}

impl<T> EmptyVal for KSlice<T> {
    const EMPTY: Self = KSlice::empty();
}

impl EmptyVal for ExtendedOptionHead {
    const EMPTY: Self = ExtendedOptionHead::ZERO;
}

macro_rules! def_option_type {
    {
        $(#[$meta:meta])*
        $vis:vis struct $opt_ty:ident (polymorphic) {
            $($(#[$meta2:meta])* $vis2:vis $field:ident : $ty:ty),*
            $(,)?
        }
    } => {
        $(#[$meta])*
        #[repr(C, align(32))]
        #[derive(Copy, Clone)]
        $vis struct $opt_ty {
            $vis head: $crate::sys::option::ExtendedOptionHead,
            $($(#[$meta2])* $vis2 $field: $ty),*
        }

        impl $crate::sys::def_option_types::EmptyVal for $opt_ty {
            const EMPTY: Self = Self {
                head: <$crate::sys::option::ExtendedOptionHead as $crate::sys::def_option_types::EmptyVal>::EMPTY,
                $($field: $crate::sys::def_option_types::EmptyVal::EMPTY),*
            };
        }

        impl $opt_ty {
            pub const NIL: Self = <Self as $crate::sys::def_option_types::EmptyVal>::EMPTY;
        }

        #[cfg(feature = "bytemuck")]
        unsafe impl ::bytemuck::Zeroable for $opt_ty{}

        #[cfg(feature = "bytemuck")]
        unsafe impl ::bytemuck::AnyBitPattern for $opt_ty{}
    };
    {
        $(#[$meta:meta])*
        $vis:vis struct $opt_ty:ident ($uuid:literal) {
            $($(#[$meta2:meta])* $vis2:vis $field:ident : $ty:ty),*
            $(,)?
        }
    } => {
        $(#[$meta])*
        #[repr(C, align(32))]
        #[derive(Copy, Clone)]
        $vis struct $opt_ty {
            $vis head: $crate::sys::option::ExtendedOptionHead,
            $($(#[$meta2])* $vis2 $field: $ty),*
        }

        impl $crate::sys::def_option_types::EmptyVal for $opt_ty {
            const EMPTY: Self = Self {
                head: $crate::sys::option::ExtendedOptionHead {
                    ty: $crate::uuid::parse_uuid($uuid),
                    ..<$crate::sys::option::ExtendedOptionHead as $crate::sys::def_option_types::EmptyVal>::EMPTY
                },
                $($field: $crate::sys::def_option_types::EmptyVal::EMPTY),*
            };
        }

        impl $opt_ty {
            pub const NIL: Self = <Self as $crate::sys::def_option_types::EmptyVal>::EMPTY;
        }

        #[cfg(feature = "bytemuck")]
        unsafe impl ::bytemuck::Zeroable for $opt_ty{}

        #[cfg(feature = "bytemuck")]
        unsafe impl ::bytemuck::AnyBitPattern for $opt_ty{}
    };

}

macro_rules! def_option {
    {
        $(#[$meta:meta])*
        $vis:vis union $opt_ty:ident($(#[$unknown_meta:meta])* $payload_size:expr){
            $($(#[$meta2:meta])* $vis2:vis $field:ident : $ty:ty),* $(,)?
        }
    } => {
        ::paste::paste! {
            $(#[$meta])*
            #[repr(C, align(32))]
            #[derive(Copy, Clone)]
            $vis struct [<$opt_ty Unknown>] {
                $vis head: $crate::sys::option::ExtendedOptionHead,
                $vis payload: [::core::mem::MaybeUninit<u8>; $payload_size]
            }

            #[cfg(feature = "bytemuck")]
            unsafe impl ::bytemuck::Zeroable for [<$opt_ty Unknown>]{}

            #[cfg(feature = "bytemuck")]
            unsafe impl ::bytemuck::AnyBitPattern for [<$opt_ty Unknown>]{}

            $(#[$meta])*
            #[repr(C, align(32))]
            #[derive(Copy, Clone)]
            #[cfg_attr(feature = "bytemuck", derive(::bytemuck::AnyBitPattern))]
            $vis union $opt_ty {
                $(#[$unknown_meta])* $vis unknown: [<$opt_ty Unknown>],
                $($(#[$meta2])* $vis2 $field: $ty),*
            }

            const _: () = {
                assert!(::core::mem::size_of::<$opt_ty>() == ::core::mem::size_of::<[<$opt_ty Unknown>]>())
            };
        }
    };
}
