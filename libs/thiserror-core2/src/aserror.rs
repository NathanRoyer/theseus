use core2::error::Error;
// Note: this was moved into `core` on July 30, 2021
// <https://github.com/rust-lang/rust/commit/4e17994b2ca5da45f219ad09cad591fc7d31dd59>
#[cfg(feature = "std")]
use core::panic::UnwindSafe;

pub trait AsDynError<'a> {
    fn as_dyn_error(&self) -> &(dyn Error + 'a);
}

impl<'a, T: Error + 'a> AsDynError<'a> for T {
    #[inline]
    fn as_dyn_error(&self) -> &(dyn Error + 'a) {
        self
    }
}

impl<'a> AsDynError<'a> for dyn Error + 'a {
    #[inline]
    fn as_dyn_error(&self) -> &(dyn Error + 'a) {
        self
    }
}

impl<'a> AsDynError<'a> for dyn Error + Send + 'a {
    #[inline]
    fn as_dyn_error(&self) -> &(dyn Error + 'a) {
        self
    }
}

impl<'a> AsDynError<'a> for dyn Error + Send + Sync + 'a {
    #[inline]
    fn as_dyn_error(&self) -> &(dyn Error + 'a) {
        self
    }
}

#[cfg(feature = "std")]
impl<'a> AsDynError<'a> for dyn Error + Send + Sync + UnwindSafe + 'a {
    #[inline]
    fn as_dyn_error(&self) -> &(dyn Error + 'a) {
        self
    }
}
