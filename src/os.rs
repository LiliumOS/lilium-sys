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

impl AsRef<str> for OsStr {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsMut<str> for OsStr {
    fn as_mut(&mut self) -> &mut str {
        self.as_str_mut()
    }
}

impl OsStr {
    pub fn new<S: AsRef<OsStr> + ?Sized>(x: &S) -> &OsStr {
        x.as_ref()
    }

    pub fn from_mut<S: AsMut<OsStr> + ?Sized>(x: &mut S) -> &mut OsStr {
        x.as_mut()
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
