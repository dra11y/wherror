use wherror::Error;

#[derive(Error, Debug)]
pub enum Error {
    What {
        #[error("...")]
        io: std::io::Error,
    },
}

fn main() {}
