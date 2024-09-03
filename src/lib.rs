#![cfg_attr(not(feature = "std"), no_std)]
#![feature(thread_local, never_type)]
//! High and Low-level bindings to the PhantomOS kernel interfaces

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod sys;

pub mod uuid;

#[cfg(feature = "api")]
pub mod fs;
#[cfg(feature = "api")]
pub mod handle;
#[cfg(feature = "api")]
pub mod io;
#[cfg(feature = "api")]
pub mod kstr;
#[cfg(feature = "api")]
pub mod os;
#[cfg(feature = "api")]
pub mod process;
#[cfg(feature = "api")]
pub mod random;
#[cfg(feature = "api")]
pub mod result;
#[cfg(feature = "api")]
pub mod security;

#[cfg(feature = "api")]
pub mod time;

#[cfg(feature = "api")]
pub mod thread;

#[cfg(feature = "api")]
pub mod info;
