#![cfg_attr(not(feature = "std"), no_std)]
#![feature(
    thread_local,
    never_type,
    non_exhaustive_omitted_patterns_lint,
    macro_metavar_expr,
    macro_metavar_expr_concat
)]
#![cfg_attr(
    all(feature = "std", feature = "unstable-std-io_error_more"),
    feature(io_error_more, io_error_inprogress)
)]

//! High and Low-level bindings to the Lilium kernel interfaces
//!
//! ## Features
//! The following features are defined to configure available APIs:
//! * `api`: Defines safe rust wrappers for system routines
//! * `alloc`: Allows use of types that require an allocator
//! * `std`: Allows use of types and operations that require the standard library
//! * `error-enum`: Defines [`crate::result::Error`] and [`crate::result::Result<T>`] even if `api` is not enabled
//! * `raw`: Defines all submodules of [`sys::sysno`] even if the corresponding subsystem isn't enabled
//! * `core-subsys`: Enables all core subsystem features (`base`, `thread`, `io`, `process`, `debug`, and `kmgmt`)
//! * `bytemuck`: Enables [`bytemuck`] trait implementations ([`Pod`][bytemuck::Pod], [`Zeroable`][bytemuck::Zeroable], [`AnyBitPattern`][bytemuck::AnyBitPattern], and [`NoUninit`][bytemuck::NoUninit])
//! * `uuid`: Enables bidirectional conversions to and from [`crate::uuid::Uuid`] and [`::uuid::Uuid`]
//!
//! The following features are enabled by default:
//! * `api`
//! * `core-subsys`
//!
//! ### Subsystem Features
//!
//! Subsystems use features to control what routines are available.
//!
//! * `core`
//! * `thread`
//! * `io`
//! * `process`
//! * `debug`
//! * `kmgmt`
//!
//! The following features for routines defined purely by the USI are also provided:
//! * `libc`: Exposes routines defined by `libc`
//! * `rtld`: Exposes routines defined by `libusi-rtld.so`
//!
//! If the `link-usi` feature is enabled, the corresponding modules are linked by a `#[link]` attribute.
//!
//! ### `unstable`` Features
//!
//! Features starting with `unstable` require nightly rust support and are exempt from semver guarantees:
//! * `unstable-std-io_error_more`: When the `std` feature is also enabled, enables conversion from [`crate::result::Error`] to [`std::io::Error`]
//!

#[cfg(feature = "alloc")]
extern crate alloc;

#[doc(hidden)]
pub use paste as _paste;

pub mod sys;

pub mod misc;

pub mod uuid;

#[cfg(all(feature = "api", feature = "thread"))]
pub mod atomic;

#[cfg(all(feature = "api", feature = "io"))]
pub mod fs;
#[cfg(feature = "api")]
pub mod handle;
#[cfg(feature = "api")]
pub mod io;
#[cfg(feature = "api")]
pub mod kstr;
#[cfg(feature = "api")]
pub mod os;
#[cfg(all(feature = "api", feature = "process"))]
pub mod process;
#[cfg(all(feature = "api", feature = "io"))]
pub mod random;
#[cfg(feature = "error-enum")]
pub mod result;
#[cfg(feature = "api")]
pub mod security;

#[cfg(all(feature = "api", feature = "thread"))]
pub mod sync;

#[cfg(feature = "api")]
pub mod time;

#[cfg(all(feature = "api", feature = "thread"))]
pub mod thread;

#[cfg(feature = "api")]
pub mod info;
