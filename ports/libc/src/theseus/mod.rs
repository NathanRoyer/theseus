//! libc types for [Theseus OS](https://github.com/theseus-os/Theseus/)
//!
//! Ported by Kevin Boos <kevinaboos@gmail.com>.
//! Most typedefs are borrowed from this crate's `src/unix/mod.rs`
//! or from `src/unix/linux_like/linux/uclibc/x86_64/mod.rs`.

pub type c_schar = i8;
pub type c_uchar = u8;
pub type c_short = i16;
pub type c_ushort = u16;
pub type c_int = i32;
pub type c_uint = u32;
pub type c_float = f32;
pub type c_double = f64;
pub type c_longlong = i64;
pub type c_ulonglong = u64;
pub type intmax_t = i64;
pub type uintmax_t = u64;

pub type size_t = usize;
pub type ptrdiff_t = isize;
pub type intptr_t = isize;
pub type uintptr_t = usize;
pub type ssize_t = isize;

pub type c_long = i64;
pub type c_ulong = u64;
pub type off_t = i64;

pub type wint_t = u32;


cfg_if! {
    if #[cfg(target_arch = "x86_64")] {
        mod x86_64;
        pub use self::x86_64::*;
    } else {
        // Unknown target_arch
    }
}

// Note: this was copied from other platforms.
cfg_if! {
if #[cfg(libc_core_cvoid)] {
        pub use ffi::c_void;
    } else {
        // Use repr(u8) as LLVM expects `void*` to be the same as `i8*` to help
        // enable more optimization opportunities around it recognizing things
        // like malloc/free.
        #[repr(u8)]
        #[allow(missing_copy_implementations)]
        #[allow(missing_debug_implementations)]
        pub enum c_void {
            // Two dummy variants so the #[repr] attribute can be used.
            #[doc(hidden)]
            __variant1,
            #[doc(hidden)]
            __variant2,
        }
    }
}



#[cfg_attr(feature = "extra_traits", derive(Debug))]
pub enum FILE {}
impl Copy for FILE {}
impl Clone for FILE {
    fn clone(&self) -> FILE {
        *self
    }
}
#[cfg_attr(feature = "extra_traits", derive(Debug))]
pub enum fpos_t {} 
impl Copy for fpos_t {}
impl Clone for fpos_t {
    fn clone(&self) -> fpos_t {
        *self
    }
}


mod mm {
    use super::*;

    pub const PROT_NONE: c_int = 0;
    pub const PROT_READ: c_int = 1;
    pub const PROT_WRITE: c_int = 2;
    pub const PROT_EXEC: c_int = 4;

    pub const MAP_FILE: c_int = 0x0000;
    pub const MAP_SHARED: c_int = 0x0001;
    pub const MAP_PRIVATE: c_int = 0x0002;
    pub const MAP_ANON: c_int = 0x0020;
    pub const MAP_ANONYMOUS: c_int = MAP_ANON;
    pub const MAP_FIXED: c_int = 0x0010;
    pub const MAP_FAILED: *mut c_void = !0 as _;

    pub const MS_ASYNC: c_int = 0x0001;
    pub const MS_INVALIDATE: c_int = 0x0002;
    pub const MS_SYNC: c_int = 0x0004;

    extern "C" {
        pub fn mlock(addr: *const c_void, len: size_t) -> c_int;
        pub fn munlock(addr: *const c_void, len: size_t) -> c_int;
        pub fn mlockall(flags: c_int) -> c_int;
        pub fn munlockall() -> c_int;
        pub fn mmap(
            addr: *mut c_void,
            len: size_t,
            prot: c_int,
            flags: c_int,
            fd: c_int,
            offset: off_t,
        ) -> *mut c_void;
        pub fn munmap(addr: *mut c_void, len: size_t) -> c_int;
    }
}
pub use self::mm::*;


mod printf {
    use super::*;
    
    extern "C" {
        // `FILE` not yet supported
        // pub fn fprintf(stream: *mut FILE, format: *const c_char, ...) -> c_int;
        pub fn printf(format: *const c_char, ...) -> c_int;
        pub fn snprintf(s: *mut c_char, n: size_t, format: *const c_char, ...) -> c_int;
        pub fn sprintf(s: *mut c_char, format: *const c_char, ...) -> c_int;
    }
}
pub use self::printf::*;
