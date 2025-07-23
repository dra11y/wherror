use wherror::Error;

#[derive(Error, Debug)]
#[error(auto)]
pub enum MyError {
    First,
    Second,
}

fn main() {}
