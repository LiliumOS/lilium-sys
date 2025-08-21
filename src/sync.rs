#[cfg(feature = "sync-mutex")]
pub mod mutex;

#[cfg(feature = "sync-mutex")]
pub use mutex::{Mutex, RwLock};

pub mod event;

#[cfg(feature = "once-lock")]
mod once;
