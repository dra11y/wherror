use wherror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Other(anyhow::Error, String),
}

fn main() {}
