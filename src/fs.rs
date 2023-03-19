use core::{str::Split, ops::Deref, borrow::Borrow};

use alloc::{string::{ToString, String},vec::Vec, borrow::Cow};

use crate::{sys::{kstr::{KStrCPtr, KStrPtr}, fs::ReadSymbolicLink, handle::HandlePtr}, result::Error};



#[repr(transparent)]
#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Path(str);


pub enum Component<'a>{
    RealPath(&'a Path),
    Root,
    CurDir,
    ParentDir,
}

pub struct Components<'a>{
    next_is_root: bool,
    split: Split<'a,char>
}

impl<'a> Iterator for Components<'a>{
    type Item = Component<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let s = self.split.next()?;
        if core::mem::take(&mut self.next_is_root){
            Some(Component::Root)
        }else if s=="."{
            Some(Component::CurDir)
        }else if s==".."{
            Some(Component::ParentDir)
        }else{
            Some(Component::RealPath(Path::new(s)))
        }
    }
}

impl AsRef<Path> for str{
    fn as_ref(&self) -> &Path {
        Path::new(self)
    }
}

impl AsRef<Path> for String{
    fn as_ref(&self) -> &Path {
        Path::new(self)
    }
}

impl AsRef<Path> for Cow<'_,str>{
    fn as_ref(&self) -> &Path {
        Path::new(self)
    }
}


impl Path{
    pub fn new<S: AsRef<str> + ?Sized>(s: &S) -> &Self{
        let s = s.as_ref();
        unsafe{&*(s as *const str as *const Path)}
    }

    pub const fn as_str(&self) -> &str{
        &self.0
    }

    pub fn file_name(&self) -> Option<&Path>{
        self.0.rsplit_once("/")
            .map(|(_,b)|b)
            .map(Path::new)
    }

    pub fn components(&self) -> Components{
        let next_is_root = self.0.starts_with("/");
        Components { next_is_root, split: self.0.split('/') }
    }

    pub fn to_path_buf(&self) -> PathBuf{
        PathBuf(self.0.to_string())
    }

    pub const fn len(&self) -> usize{
        self.0.len()
    }

    pub const fn to_kstr_raw(&self) -> KStrCPtr{
        KStrCPtr::from_str(self.as_str())
    }

}

impl core::fmt::Display for Path{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result{
        self.0.fmt(f)
    }
}

impl AsRef<str> for Path{
    fn as_ref(&self) -> &str{
        &self.0
    }
}

impl AsRef<[u8]> for Path{
    fn as_ref(&self) -> &[u8]{
        self.0.as_bytes()
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct PathBuf(String);

impl<S: AsRef<Path>+?Sized> From<&S> for PathBuf{
    fn from(s: &S) -> Self{
        Self(s.as_ref().as_str().to_string())
    }
}


impl PathBuf{
    pub const fn new() -> Self{
        Self(String::new())
    } 

    pub const fn from_string(s: String) -> Self{
        Self(s)
    }

    pub fn into_string(self) -> String{
        self.0
    }

    pub fn as_path(&self) -> &Path{
        Path::new(&self.0)
    }
}

impl Deref for PathBuf{
    type Target = Path;
    fn deref(&self) -> &Self::Target {
        self.as_path()
    }
}

impl AsRef<Path> for PathBuf{
    fn as_ref(&self) -> &Path{
        self.as_path()
    }
}

impl Borrow<Path> for PathBuf{
    fn borrow(&self) -> &Path{
        self.as_path()
    }
}

pub fn read_symlink<P: AsRef<Path>>(path: P) -> crate::result::Result<PathBuf>{
    let path = path.as_ref();

    let mut buf = Vec::<u8>::with_capacity(256);

    let mut kstr = KStrPtr{
        str_ptr: buf.as_mut_ptr() as *mut i8,
        len: 256,
    };

    match crate::result::Error::from_code(unsafe{ReadSymbolicLink(HandlePtr::null(), KStrCPtr::from_str(path.as_ref()), &mut kstr)}){
        Ok(()) => {
            if kstr.len > 256{
                buf.reserve(kstr.len as usize);
                kstr.str_ptr = buf.as_mut_ptr() as *mut i8;
                crate::result::Error::from_code(unsafe{ReadSymbolicLink(HandlePtr::null(), KStrCPtr::from_str(path.as_ref()), &mut kstr)})?;
            }
        }
        Err(Error::InsufficientLength) => {
            buf.reserve(kstr.len as usize);
            kstr.str_ptr = buf.as_mut_ptr() as *mut i8;
            crate::result::Error::from_code(unsafe{ReadSymbolicLink(HandlePtr::null(), KStrCPtr::from_str(path.as_ref()), &mut kstr)})?;
        }
        Err(e) => return Err(e)
    }

    // SAFETY:
    // The kernel wrote exactly kstr.len bytes
    unsafe{buf.set_len(kstr.len as usize);}

    buf.shrink_to_fit();

    // SAFETY:
    // The Lillium kernel guarantees that a non-truncated strings returned from kernel space to userspace are valid UTF-8 
    let st = unsafe{String::from_utf8_unchecked(buf)};
    
    Ok(PathBuf(st))
}