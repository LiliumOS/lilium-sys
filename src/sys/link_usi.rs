#[cfg_attr(feature = "base", link(name = "usi-base"))]
#[cfg_attr(feature = "thread", link(name = "usi-thread"))]
#[cfg_attr(feature = "io", link(name = "usi-io"))]
#[cfg_attr(feature = "process", link(name = "usi-process"))]
#[cfg_attr(feature = "debug", link(name = "usi-debug"))]
#[cfg_attr(feature = "kmgmt", link(name = "usi-kmgmt"))]
#[cfg_attr(feature = "vti", link(name = "usi-vti"))]
#[cfg_attr(feature = "libc", link(name = "c"))]
#[cfg_attr(feature = "rtld", link(name = "usi-rtld"))]
unsafe extern "C" {}
