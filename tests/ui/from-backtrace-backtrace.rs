// https://github.com/dtolnay/thiserror/issues/163

use std::backtrace::Backtrace;
use wherror::Error;

#[derive(Error, Debug)]
#[error("...")]
pub struct Error(
    #[from]
    #[backtrace]
    std::io::Error,
    Backtrace,
);

fn main() {}
