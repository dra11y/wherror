use wherror::Error;

#[derive(Error, Debug)]
#[error("".yellow)]
pub struct ArgError;

fn main() {}
