// https://github.com/dtolnay/thiserror/issues/163

#![feature(backtrace)]

use core2::backtrace::Backtrace;
use thiserror::Error;

#[derive(Error, Debug)]
#[error("...")]
pub struct Error(#[from] #[backtrace] core2::io::Error, Backtrace);

fn main() {}
