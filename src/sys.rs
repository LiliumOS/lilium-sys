//! Low-level interfaces to Lilium
#![allow(unexpected_cfgs)] // Clever-ISA will be supported by lccc

#[macro_use]
pub(crate) mod def_option_types;

pub mod error;
pub mod handle;
pub mod kstr;
pub mod option;
pub mod result;
pub mod sysno;

pub mod config;
pub mod except;
pub mod info;
pub mod permission;

pub mod thread;

pub mod device;
pub mod fs;
pub mod io;
pub mod random;
pub mod socket;
pub mod time;

pub mod ipc;
pub mod isolation;
pub mod process;

pub mod debug;

pub mod kmgmt;

#[cfg(any(feature = "libc", doc))]
pub mod signal;

#[cfg(any(feature = "vti", doc))]
pub mod vti;

#[cfg(all(feature = "link-usi", target_os = "lilium"))]
mod link_usi;

pub mod auxv;
