use core::{
    borrow::Borrow,
    ffi::{c_long, c_void},
    ops::Deref,
    str::Split,
};

use alloc::{
    borrow::Cow,
    string::{String, ToString},
    vec::Vec,
};

use core::mem::MaybeUninit;

use crate::{
    handle::{AsHandle, OwnedHandle, SharedHandle},
    result::{Error, Result},
    sys::{
        fs::{self as sys, DirectoryInfo, DirectoryNext, DirectoryRead, FileHandle},
        handle::{Handle, HandlePtr},
        kstr::{KCSlice, KStrCPtr, KStrPtr},
        result::errors::DOES_NOT_EXIST,
    },
    thread::TlsKey,
    time::{Duration, SystemClock, TimePoint},
    uuid::Uuid,
};

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct OwnedFile(OwnedHandle<FileHandle>);

impl OwnedFile {
    pub const unsafe fn from_handle(hdl: HandlePtr<FileHandle>) -> OwnedFile { unsafe {
        Self(OwnedHandle::take_ownership(hdl))
    }}

    pub fn as_raw(&self) -> HandlePtr<FileHandle> {
        self.0.as_raw()
    }
}

unsafe impl<'a> AsHandle<'a, FileHandle> for &'a OwnedFile {
    fn as_handle(&self) -> HandlePtr<FileHandle> {
        self.0.as_raw()
    }
}

impl Clone for OwnedFile {
    fn clone(&self) -> Self {
        let mut ptr = MaybeUninit::uninit();
        unsafe {
            Error::from_code(sys::DuplicateFile(ptr.as_mut_ptr(), self.0.as_raw())).unwrap();
        }

        Self(unsafe { OwnedHandle::take_ownership(ptr.assume_init()) })
    }
}

#[derive(Debug)]
pub struct SharedFile(SharedHandle<FileHandle>);

#[repr(transparent)]
#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Path(str);

pub enum Component<'a> {
    RealPath(&'a Path),
    Root,
    CurDir,
    ParentDir,
}

impl<'a> Component<'a> {
    pub const fn as_str(&self) -> &'a str {
        match self {
            Component::RealPath(p) => p.as_str(),
            Component::Root => "/",
            Component::CurDir => ".",
            Component::ParentDir => "..",
        }
    }
}

pub struct Components<'a> {
    next_is_root: bool,
    split: Split<'a, char>,
}

impl<'a> Iterator for Components<'a> {
    type Item = Component<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let s = self.split.next()?;
        if core::mem::take(&mut self.next_is_root) {
            Some(Component::Root)
        } else if s == "." {
            Some(Component::CurDir)
        } else if s == ".." {
            Some(Component::ParentDir)
        } else {
            Some(Component::RealPath(Path::new(s)))
        }
    }
}

impl AsRef<Path> for str {
    fn as_ref(&self) -> &Path {
        Path::new(self)
    }
}

impl AsRef<Path> for String {
    fn as_ref(&self) -> &Path {
        Path::new(self)
    }
}

impl AsRef<Path> for Cow<'_, str> {
    fn as_ref(&self) -> &Path {
        Path::new(self)
    }
}

impl Path {
    pub fn new<S: AsRef<str> + ?Sized>(s: &S) -> &Self {
        let s = s.as_ref();
        unsafe { &*(s as *const str as *const Path) }
    }

    pub const fn as_str(&self) -> &str {
        &self.0
    }

    pub fn file_name(&self) -> Option<&Path> {
        self.0.rsplit_once("/").map(|(_, b)| b).map(Path::new)
    }

    pub fn components(&self) -> Components {
        let next_is_root = self.0.starts_with("/");
        Components {
            next_is_root,
            split: self.0.split('/'),
        }
    }

    pub fn to_path_buf(&self) -> PathBuf {
        PathBuf(self.0.to_string())
    }

    pub const fn len(&self) -> usize {
        self.0.len()
    }

    pub const fn to_kstr_raw(&self) -> KStrCPtr {
        KStrCPtr::from_str(self.as_str())
    }
}

impl core::fmt::Display for Path {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        self.0.fmt(f)
    }
}

impl AsRef<str> for Path {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl AsRef<[u8]> for Path {
    fn as_ref(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct PathBuf(String);

impl<S: AsRef<Path> + ?Sized> From<&S> for PathBuf {
    fn from(s: &S) -> Self {
        Self(s.as_ref().as_str().to_string())
    }
}

impl PathBuf {
    pub const fn new() -> Self {
        Self(String::new())
    }

    pub const fn from_string(s: String) -> Self {
        Self(s)
    }

    pub fn into_string(self) -> String {
        self.0
    }

    pub fn as_path(&self) -> &Path {
        Path::new(&self.0)
    }
}

impl Deref for PathBuf {
    type Target = Path;
    fn deref(&self) -> &Self::Target {
        self.as_path()
    }
}

impl AsRef<Path> for PathBuf {
    fn as_ref(&self) -> &Path {
        self.as_path()
    }
}

impl Borrow<Path> for PathBuf {
    fn borrow(&self) -> &Path {
        self.as_path()
    }
}

pub fn read_link<P: AsRef<Path>>(path: P) -> crate::result::Result<PathBuf> {
    let path = path.as_ref();

    let mut buf = Vec::<u8>::with_capacity(256);

    let mut kstr = KStrPtr {
        str_ptr: buf.as_mut_ptr(),
        len: 256,
    };

    match crate::result::Error::from_code(unsafe {
        sys::ReadSymbolicLink(
            HandlePtr::null(),
            KStrCPtr::from_str(path.as_ref()),
            &mut kstr,
        )
    }) {
        Ok(()) => {
            if kstr.len > 256 {
                buf.reserve(kstr.len as usize);
                kstr.str_ptr = buf.as_mut_ptr();
                crate::result::Error::from_code(unsafe {
                    sys::ReadSymbolicLink(
                        HandlePtr::null(),
                        KStrCPtr::from_str(path.as_ref()),
                        &mut kstr,
                    )
                })?;
            }
        }
        Err(Error::InsufficientLength) => {
            buf.reserve(kstr.len as usize);
            kstr.str_ptr = buf.as_mut_ptr();
            crate::result::Error::from_code(unsafe {
                sys::ReadSymbolicLink(
                    HandlePtr::null(),
                    KStrCPtr::from_str(path.as_ref()),
                    &mut kstr,
                )
            })?;
        }
        Err(e) => return Err(e),
    }

    // SAFETY:
    // The kernel wrote exactly kstr.len bytes
    unsafe {
        buf.set_len(kstr.len as usize);
    }

    buf.shrink_to_fit();

    // SAFETY:
    // The Lillium kernel guarantees that a non-truncated strings returned from kernel space to userspace are valid UTF-8
    let st = unsafe { String::from_utf8_unchecked(buf) };

    Ok(PathBuf(st))
}

pub fn hard_link<P: AsRef<Path>, Q: AsRef<Path>>(
    original: P,
    link: Q,
) -> crate::result::Result<()> {
    crate::result::Error::from_code(unsafe {
        sys::CreateHardLink(
            core::ptr::null_mut(),
            HandlePtr::null(),
            KStrCPtr::from_str(link.as_ref().as_str()),
            HandlePtr::null(),
            KStrCPtr::from_str(original.as_ref().as_str()),
        )
    })
}

pub fn weak_link<P: AsRef<Path>, Q: AsRef<Path>>(
    original: P,
    link: Q,
) -> crate::result::Result<()> {
    crate::result::Error::from_code(unsafe {
        crate::sys::fs::CreateWeakLink(
            core::ptr::null_mut(),
            HandlePtr::null(),
            KStrCPtr::from_str(link.as_ref().as_str()),
            HandlePtr::null(),
            KStrCPtr::from_str(original.as_ref().as_str()),
        )
    })
}

pub fn symlink<P: AsRef<Path>, Q: AsRef<Path>>(original: P, link: Q) -> crate::result::Result<()> {
    crate::result::Error::from_code(unsafe {
        crate::sys::fs::CreateSymbolicLink(
            HandlePtr::null(),
            KStrCPtr::from_str(link.as_ref().as_str()),
            KStrCPtr::from_str(original.as_ref().as_str()),
        )
    })
}

pub fn create_dir_all<P: AsRef<Path>>(path: P) -> crate::result::Result<()> {
    let path = path.as_ref();

    let mut cur_base = HandlePtr::null();

    for seg in path.components() {
        loop {
            match crate::result::Error::from_code(unsafe {
                sys::OpenFile(
                    &mut cur_base,
                    cur_base,
                    KStrCPtr::from_str(seg.as_str()),
                    &sys::FileOpenOptions {
                        stream_override: KStrCPtr::empty(),
                        access_mode: sys::ACCESS_READ,
                        op_mode: sys::OP_DIRECTORY_ACCESS,
                        blocking_mode: sys::MODE_BLOCKING,
                        create_acl: HandlePtr::null(),
                        extended_options: KCSlice::empty(),
                    },
                )
            }) {
                Ok(()) => break,
                Err(crate::result::Error::DoesNotExist) => {
                    match crate::result::Error::from_code(unsafe {
                        sys::CreateDirectory(
                            &mut cur_base,
                            cur_base,
                            KStrCPtr::from_str(seg.as_str()),
                            HandlePtr::null(),
                        )
                    }) {
                        Ok(()) => break,
                        Err(crate::result::Error::AlreadyExists) => continue,
                        Err(e) => return Err(e),
                    }
                }
                Err(e) => return Err(e),
            }
        }
    }

    Ok(())
}

pub struct DirIterator {
    dirhdl: HandlePtr<FileHandle>,
    base_path: PathBuf,
    state: *mut c_void,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct FileType(u16);

impl FileType {
    pub fn is_file(&self) -> bool {
        self.0 == 0
    }

    pub fn is_dir(&self) -> bool {
        self.0 == 1
    }

    pub fn is_symlink(&self) -> bool {
        self.0 == 2
    }

    pub fn is_fifo(&self) -> bool {
        self.0 == 3
    }

    pub fn is_socket(&self) -> bool {
        self.0 == 4
    }

    pub fn is_block_device(&self) -> bool {
        self.0 == 5
    }

    pub fn is_char_device(&self) -> bool {
        self.0 == 6
    }

    pub fn is_custom(&self) -> bool {
        self.0 == 65535
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
#[non_exhaustive]
pub enum MetadataEntry {
    AccessTime(TimePoint<SystemClock>),
    CreationTime(TimePoint<SystemClock>),
    ModificationTime(TimePoint<SystemClock>),
    CreatedBy(String),

    Unknown(String),
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Metadata {
    entries: Vec<MetadataEntry>,
    file_type: FileType,
    custom_ty: Option<String>,
    permissions: Permissions,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Permissions(OwnedFile);

impl Permissions {
    pub fn readonly(&self) -> bool {
        unsafe {
            sys::AclTestPermission(
                self.0.as_raw(),
                KStrCPtr::from_str("Write"),
                KStrCPtr::empty(),
            ) == 0
        }
    }

    pub fn set_readonly(&mut self, _readonly: bool) {
        // No-op
        // Behaviour of this function is unclear on Lilium, so the most sensible behaviour is doing literally nothing
    }
}

impl Permissions {
    pub fn empty() -> Result<Self> {
        let mut hdl = MaybeUninit::uninit();

        Error::from_code(unsafe { sys::CreateAcl(hdl.as_mut_ptr()) })?;

        Ok(Self(unsafe { OwnedFile::from_handle(hdl.assume_init()) }))
    }

    pub fn default_acl() -> Result<Self> {
        let mut hdl = MaybeUninit::uninit();

        Error::from_code(unsafe { sys::DefaultAcl(hdl.as_mut_ptr()) })?;

        Ok(Self(unsafe { OwnedFile::from_handle(hdl.assume_init()) }))
    }

    pub unsafe fn from_file_handle(base: HandlePtr<FileHandle>) -> Result<Self> {
        let mut hdl = MaybeUninit::uninit();

        Error::from_code(unsafe { sys::CopyAcl(hdl.as_mut_ptr(), base) })?;

        Ok(Self(unsafe { OwnedFile::from_handle(hdl.assume_init()) }))
    }

    /// Tests whether the current thread has the given `name`d permission in the ACL represented by this object.
    ///
    /// This takes into account both the DACL and legacy security descriptor.
    ///
    pub fn test_permission(&self, name: &str) -> Result<bool> {
        match Error::from_code(unsafe {
            sys::AclTestPermission(self.0.as_raw(), KStrCPtr::from_str(name), KStrCPtr::empty())
        }) {
            Ok(()) => Ok(true),
            Err(Error::Permission) => Ok(false),
            Err(e) => Err(e),
        }
    }

    pub fn test_stream_permission(&self, name: &str, stream: &str) -> Result<bool> {
        match Error::from_code(unsafe {
            sys::AclTestPermission(
                self.0.as_raw(),
                KStrCPtr::from_str(name),
                KStrCPtr::from_str(stream),
            )
        }) {
            Ok(()) => Ok(true),
            Err(Error::Permission) => Ok(false),
            Err(e) => Err(e),
        }
    }

    pub fn legacy_mode(&self) -> Option<u32> {
        let mode = unsafe { sys::AclLegacyMode(self.0.as_raw()) };

        match Error::from_code(mode) {
            Ok(()) => Some(mode as u32),
            Err(Error::DoesNotExist) => None,
            Err(e) => Err(e).unwrap(),
        }
    }

    pub fn legacy_uid(&self) -> Option<u32> {
        let mode = unsafe { sys::AclLegacyUid(self.0.as_raw()) };

        match Error::from_code(mode) {
            Ok(()) => Some(mode as u32),
            Err(Error::DoesNotExist) => None,
            Err(e) => Err(e).unwrap(),
        }
    }

    pub fn legacy_gid(&self) -> Option<u32> {
        let mode = unsafe { sys::AclLegacyGid(self.0.as_raw()) };

        match Error::from_code(mode) {
            Ok(()) => Some(mode as u32),
            Err(Error::DoesNotExist) => None,
            Err(e) => Err(e).unwrap(),
        }
    }

    pub fn set_legacy_mode(&mut self, mode: u32) -> Result<()> {
        Error::from_code(unsafe { sys::AclSetLegacyMode(self.0.as_raw(), mode) })
    }

    pub fn set_legacy_uid(&mut self, uid: u32) -> Result<()> {
        Error::from_code(unsafe { sys::AclSetLegacyUid(self.0.as_raw(), uid as c_long) })
    }

    pub fn set_legacy_gid(&mut self, gid: u32) -> Result<()> {
        Error::from_code(unsafe { sys::AclSetLegacyUid(self.0.as_raw(), gid as c_long) })
    }

    pub fn set_owner(&mut self, uuid: Uuid) -> Result<()> {
        Error::from_code(unsafe { sys::SetObjectOwner(self.0.as_raw(), &uuid) })
    }

    pub fn take_ownership(&mut self) -> Result<()> {
        let mut uuid = MaybeUninit::uninit();
        Error::from_code(unsafe {
            crate::sys::permission::GetPrimaryPrincipal(HandlePtr::null(), uuid.as_mut_ptr())
        })?;

        let uuid = unsafe { uuid.assume_init() };

        Error::from_code(unsafe { sys::SetObjectOwner(self.0.as_raw(), &uuid) })
    }

    /// Determines the owner of the file represented by this [`Permissions`] structure.
    ///
    /// Returns `Some(principal)` if the owner is determined, or `None` if no owner is present.
    ///
    /// A permission object may have no owner in one of many ways:
    /// * The object has an ACL, and no `ObjectOwner` row exists,
    /// * The object has no ACL but has a legacy security descriptor, and the uid is not present in the principal map for the current filesystem, nor the default principal map,
    /// * The object has neither an ACL nor a legacy security descriptor (this is possible on a [`Permissions`] object obtained from a filesystem without either ACLs or legacy permissions when an override is not provided), or
    /// * The Object does not represent an ACL or legacy descriptor, nor is an ACL or legacy descriptor accesible via the handle the [`Permission`] object wraps.
    ///
    ///
    /// This always returns a principal in the enhanced permission space.
    pub fn owner(&self) -> Option<Uuid> {
        let mut uuid = MaybeUninit::uninit();
        match Error::from_code(unsafe { sys::ObjectOwner(self.0.as_raw(), uuid.as_mut_ptr()) }) {
            Ok(()) => Some(unsafe { uuid.assume_init() }),
            Err(Error::DoesNotExist) => None,
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }
}

pub struct DirEntry(OwnedFile);
