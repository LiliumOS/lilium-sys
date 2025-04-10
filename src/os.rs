#[repr(transparent)]
pub struct OsStr(str);

impl AsRef<OsStr> for str {
    fn as_ref(&self) -> &OsStr {
        OsStr::from_str(self)
    }
}

impl AsMut<OsStr> for str {
    fn as_mut(&mut self) -> &mut OsStr {
        OsStr::from_str_mut(self)
    }
}

impl OsStr {
    pub fn new<S: AsRef<OsStr> + ?Sized>(x: &S) -> &OsStr {
        x.as_ref()
    }

    pub fn from_mut<S: AsMut<OsStr> + ?Sized>(x: &mut S) -> &mut OsStr {
        x.as_mut()
    }

    pub fn display(&self) -> Display {
        Display(&self.0)
    }
}

pub struct Display<'a>(&'a str);

impl core::fmt::Display for Display<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.0)
    }
}

impl OsStr {
    #[inline]
    pub fn from_str(x: &str) -> &OsStr {
        unsafe { &*(x as *const str as *const OsStr) }
    }

    #[inline]
    pub fn from_str_mut(x: &mut str) -> &mut OsStr {
        unsafe { &mut *(x as *mut str as *mut OsStr) }
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    #[inline]
    pub fn as_str_mut(&mut self) -> &mut str {
        &mut self.0
    }
}

#[cfg(feature = "alloc")]
mod alloc {
    use ::alloc::string::String;
    pub struct OsString(String);
}

#[cfg(feature = "alloc")]
pub use alloc::*;
