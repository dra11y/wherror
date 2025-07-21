use wherror::Error;

#[derive(Error, Debug)]
#[error("{_}")]
pub struct Error;

fn main() {}
