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
            Error::Killed => todo!(),
            Error::UnsupportedOperation => todo!(),
            #[cfg(feature = "unstable-std-io_error_more")]
            Error::Pending => ErrorKind::InProgress,
            #[cfg(not(feature = "unstable-std-io_error_more"))]
            Error::Pending => ErrorKind::Other,
            Error::DoesNotExist => todo!(),
            Error::AlreadyExists => todo!(),
            Error::UnknownDevice => todo!(),
            Error::WouldBlock => todo!(),
            Error::DeviceFull => todo!(),
            Error::DeviceUnavailable => todo!(),
            Error::LinkResolutionLoop => todo!(),
            Error::ClosedRemotely => todo!(),
            Error::ConnectionInterrupted => todo!(),
            Error::Signaled => todo!(),
            Error::MappingInaccessible => todo!(),
            Error::PrivilegeCheckFailed => ErrorKind::PermissionDenied,

            _ => ErrorKind::Other,
        }
    }
}
