use thiserror_core2::Error;

#[derive(Error, Debug)]
pub enum ErrorEnum {
    Confusing {
        #[source]
        a: core2::io::Error,
        #[source]
        b: anyhow::Error,
    },
}

fn main() {}
