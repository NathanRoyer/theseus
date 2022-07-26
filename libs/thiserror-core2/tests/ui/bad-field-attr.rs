use thiserror_core2::Error;

#[derive(Error, Debug)]
#[error(transparent)]
pub struct Error(#[error(transparent)] core2::io::Error);

fn main() {}
