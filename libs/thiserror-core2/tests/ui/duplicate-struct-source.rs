use thiserror_core2::Error;

#[derive(Error, Debug)]
pub struct ErrorStruct {
    #[source]
    a: core2::io::Error,
    #[source]
    b: anyhow::Error,
}

fn main() {}
