#![deny(deprecated, clippy::all, clippy::pedantic)]

use thiserror_core2::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[deprecated]
    #[error("...")]
    Deprecated,
}
