#![no_std]
//! High and Low-level bindings to the PhantomOS kernel interfaces

extern crate alloc;


pub mod sys;
pub mod fs;
pub mod result;
pub mod kstr;
pub mod handle;
pub mod security;
pub mod uuid;
pub mod random;