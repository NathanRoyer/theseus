use thiserror_core2::Error;

#[derive(Error, Debug)]
#[error(transparent)]
pub struct Error(#[source] anyhow::Error);

fn main() {}
