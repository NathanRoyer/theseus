use thiserror_core2::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("...")]
    A(usize),
    B(usize),
}

fn main() {}
