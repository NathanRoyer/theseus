use thiserror_core2::Error;

#[derive(Error, Debug)]
#[error(transparent)]
pub struct Error {
    inner: anyhow::Error,
    what: String,
}

fn main() {}
