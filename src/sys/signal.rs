use core::{ffi::c_void, mem::MaybeUninit};

use super::{
    handle::{Handle, HandlePtr},
    process::ProcessHandle,
    result::SysResult,
    thread::ThreadHandle,
};

/// The type for the handler parameter of the signal function.
/// Note that per [`signal`], uses of this alias are expected to contain initialized memory only.
pub type sig_t = MaybeUninit<unsafe extern "system" fn(i32) -> ()>;

pub const SIG_IGN: sig_t = unsafe { core::mem::transmute(-1isize) };
pub const SIG_DFL: sig_t = unsafe { core::mem::transmute(-2isize) };

unsafe extern "system" {
    /// Updates the signal handler to the specified  action, and returns the current action.
    /// Calling this function with a `null` (all 0 bytes) instead queries the handler without changing it
    ///
    /// # Safety
    /// `action` must be a valid function pointer, null, or one of the `SIG_` constants (`SIG_IGN` or `SIG_DFL`).
    /// In particular, `hdl` must not be uninitialized memory (but can be a function pointer with a broken validity or safety invariant)
    /// The return value is guaranteed to be one of the above.
    ///
    /// # Interaction with exceptions API
    /// Signals in Lilium are implemented as a wrapper arround exceptions.
    /// In particular, `signal` is implemented by use of an exception hook in the in the standard usi libc.
    /// The hook is installed at the latest the first time `signal` is called in a process.
    /// The signal handler has all permissions that an exception hook (not exception handler) has, with the following caveats:
    /// * It is valid to synchronously raise an exception in them. However, any hooks other than the hook installed by `signal` is
    pub unsafe fn signal(sig: i32, action: sig_t) -> sig_t;

    /// Raises the specified signal synchronously.
    ///
    /// ## Behaviour
    /// This acts as though it calls [`ExceptHandleSynchronous`][super::except::ExceptHandleSynchronous] with the corresponding exception type.
    /// Per ISO C `signal`, `
    pub safe fn raise(sig: i32);
}
