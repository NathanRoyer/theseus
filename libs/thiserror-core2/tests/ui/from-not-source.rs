use thiserror_core2::Error;

#[derive(Debug, Error)]
pub struct Error {
    #[source]
    source: core2::io::Error,
    #[from]
    other: anyhow::Error,
}

fn main() {}
