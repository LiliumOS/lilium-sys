use crate::sys::result::SysResult;

pub type Result<T> = core::result::Result<T, Error>;

macro_rules! error_def{
    {$(#![$outer_meta:meta])* $($(#[$meta:meta])* #define $name:ident ($val:literal))* } => {
        paste::paste!{
            #[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
            $(#[$outer_meta])*
            #[non_exhaustive]
            pub enum Error{
                #[doc(hidden)]
                Unknown(SysResult),
                $($(#[$meta])* [<$name:camel>]),*
            }

            impl Error{
                pub const fn from_code(code: SysResult) -> Result<()>{
                    match code{
                        0..=<SysResult>::MAX => Ok(()),
                        $($val => Err(Self::[<$name:camel>]),)*
                        x => Err(Self::Unknown(x))
                    }
                }

                pub const fn into_code(self) -> SysResult {
                    match self{
                        $(Self::[<$name:camel>] => $val,)*
                        Self::Unknown(x) => x,
                    }
                }
            }
        }

    }
}
with_builtin_macros::with_builtin! {
    let $file = include_from_root!("include/errors.h") in {
        error_def!{$file}
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::Unknown(n) => f.write_fmt(format_args!("(unknown error code {})", -*n)),
            Error::Permission => f.write_str("Permission Denied"),
            Error::InvalidHandle => f.write_str("Invalid Handle Object"),
            Error::InvalidMemory => f.write_str("Invalid Memory Reference"),
            Error::Busy => f.write_str("Object Busy"),
            Error::InvalidOperation => f.write_str("Invalid Operation"),
            Error::InvalidString => f.write_str("Invalid String (Non UTF-8 data)"),
            Error::InsufficientLength => f.write_str("Insufficient Buffer Length"),
            Error::ResourceLimitExhausted => f.write_str("Resource Limit Exhausted"),
            Error::InvalidState => f.write_str("Invalid Object State"),
            Error::InvalidOption => f.write_str("Invalid Extended Option"),
            Error::InsufficientMemory => f.write_str("Insufficient Physical Memory"),
            Error::UnsupportedKernelFunction => f.write_str("Unsupported/Unknown Kernel Function"),
            Error::FinishedEnumerate => f.write_str("Enumerate Concluded"),
            Error::Timeout => f.write_str("Blocking Operation Timed Out"),
            Error::Interrupted => f.write_str("Blocking Operation Interrupted"),
            Error::Killed => f.write_str("Thread Killed"),
            Error::UnsupportedOperation => f.write_str("Unsupported Operation"),
            Error::Pending => f.write_str("Operation Pending"),
            Error::DoesNotExist => f.write_str("Object Does not Exist"),
            Error::AlreadyExists => f.write_str("Object Already Exists"),
            Error::UnknownDevice => f.write_str("Unknown Device"),
            Error::WouldBlock => f.write_str("Operation Would Block"),
            Error::DeviceFull => f.write_str("Device Full"),
            Error::DeviceUnavailable => f.write_str("Device Unavailable"),
            Error::LinkResolutionLoop => f.write_str("Link Resolution Loop"),
            Error::OrphanedObjects => f.write_str("Objects Orphaned by Operation (Prevented)"),
            Error::ClosedRemotely => f.write_str("Socket Closed Remotely"),
            Error::ConnectionInterrupted => f.write_str("Connection Interrupted"),
            Error::AddressNotAvailable => f.write_str("Address Unavailable"),
            Error::Signaled => f.write_str("Exception Recieved"),
            Error::MappingInaccessible => f.write_str("Mapping Inaccessible"),
            Error::PrivilegeCheckFailed => f.write_str("Privilege Check Failed"),
        }
    }
}

#[cfg(feature = "std")]
impl From<Error> for std::io::ErrorKind {
    fn from(value: Error) -> std::io::ErrorKind {
        use std::io::ErrorKind;
        #[deny(non_exhaustive_omitted_patterns)]
        match value {
            Error::Permission => ErrorKind::PermissionDenied,
            Error::Busy => ErrorKind::ResourceBusy,
            Error::InvalidOperation => ErrorKind::Unsupported,
            Error::InvalidString => ErrorKind::InvalidData,
            Error::InsufficientLength => ErrorKind::InvalidInput,
            Error::ResourceLimitExhausted => ErrorKind::PermissionDenied,
            Error::InvalidState => ErrorKind::Other,
            Error::InvalidHandle => ErrorKind::InvalidInput,
            Error::InvalidMemory => ErrorKind::InvalidInput,
            Error::InvalidOption => ErrorKind::Unsupported,
            Error::InsufficientMemory => ErrorKind::OutOfMemory,
            Error::UnsupportedKernelFunction => ErrorKind::Unsupported,

            Error::Timeout => ErrorKind::TimedOut,
            Error::Interrupted => ErrorKind::Interrupted,
            Error::Killed => ErrorKind::Other,
            Error::UnsupportedOperation => todo!(),
            #[cfg(feature = "unstable-std-io_error_more")]
            Error::Pending => ErrorKind::InProgress,
            Error::DoesNotExist => ErrorKind::NotFound,
            Error::AlreadyExists => ErrorKind::AlreadyExists,
            Error::UnknownDevice => ErrorKind::NotFound,
            Error::WouldBlock => ErrorKind::WouldBlock,
            Error::DeviceFull => ErrorKind::StorageFull,
            Error::DeviceUnavailable => ErrorKind::ResourceBusy,
            #[cfg(feature = "unstable-std-io_error_more")]
            Error::LinkResolutionLoop => ErrorKind::FilesystemLoop,
            Error::ClosedRemotely => ErrorKind::ConnectionAborted,
            Error::ConnectionInterrupted => ErrorKind::ConnectionReset,
            Error::MappingInaccessible => ErrorKind::InvalidInput,
            Error::PrivilegeCheckFailed => ErrorKind::PermissionDenied,

            _ => ErrorKind::Other,
        }
    }
}

#[cfg(feature = "std")]
impl From<Error> for std::io::Error {
    fn from(value: Error) -> Self {
        struct Wrapper(Error);
        impl core::fmt::Debug for Wrapper {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                self.0.fmt(f)
            }
        }
        impl core::fmt::Display for Wrapper {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                self.0.fmt(f)
            }
        }
        impl std::error::Error for Wrapper {}
        let kind = value.into();

        Self::new(kind, Wrapper(value))
    }
}

#[cfg(feature = "alloc")]
mod alloc {
    use crate::sys::error as sys;

    use ::alloc::vec::Vec;

    use super::Error;
    #[derive(Clone)]

    pub struct ContextError {
        context: Vec<sys::ErrorContextEntry>,
        errc: Error,
    }

    struct ContextEntryWrapper<'a>(&'a sys::ErrorContextEntry);

    impl core::fmt::Debug for ContextEntryWrapper<'_> {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            match unsafe { self.0.head.ty } {
                sys::ERROR_CONTEXT_TYPE_INVALID_OPTION => {
                    let ctx = unsafe { &self.0.invalid_option };
                    f.debug_struct("ErrorContextInvalidOption")
                        .field("reason_code", &format_args!("{:#010X}", ctx.reason_code))
                        .field("index", &ctx.index)
                        .field("option_id", &ctx.option_id)
                        .finish()
                }

                id => f.write_fmt(format_args!("(unknown option type {id:#?})")),
            }
        }
    }

    impl core::fmt::Debug for ContextError {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            struct DebugCtxOptionWrapper<'a>(&'a [sys::ErrorContextEntry]);

            impl core::fmt::Debug for DebugCtxOptionWrapper<'_> {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    let mut arr = f.debug_list();
                    for e in self.0 {
                        arr.entry(&ContextEntryWrapper(e));
                    }
                    arr.finish()
                }
            }

            f.debug_struct("ContextError")
                .field("errc", &self.errc)
                .field("context", &DebugCtxOptionWrapper(&self.context))
                .finish()
        }
    }

    impl core::fmt::Display for ContextError {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            self.errc.fmt(f)?;

            match self.errc {
                _ => Ok(()),
            }
        }
    }

    #[cfg(feature = "std")]
    impl std::error::Error for ContextError {}

    #[cfg(feature = "std")]
    impl From<ContextError> for std::io::Error {
        fn from(value: ContextError) -> Self {
            std::io::Error::new(value.errc.into(), value)
        }
    }
}

#[cfg(feature = "alloc")]
pub use alloc::*;
