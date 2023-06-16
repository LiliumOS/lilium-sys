#![cfg_attr(not(feature = "std"), no_std)]
#![feature(thread_local, never_type)]
//! High and Low-level bindings to the PhantomOS kernel interfaces

extern crate alloc;

pub mod fs;
pub mod handle;
pub mod io;
pub mod kstr;
pub mod process;
pub mod random;
pub mod result;
pub mod security;
pub mod sys;
pub mod uuid;
