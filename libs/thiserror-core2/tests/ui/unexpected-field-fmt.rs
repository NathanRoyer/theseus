use thiserror_core2::Error;

#[derive(Error, Debug)]
pub enum Error {
    What {
        #[error("...")]
        io: core2::io::Error,
    },
}

fn main() {}
