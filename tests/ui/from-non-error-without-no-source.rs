use wherror::Error;

/// Sounds like bad grammer, but it's not!
/// Test that #[from] NotErrorWithoutNoSource fails
/// when the user should use #[from(no_source)] NotErrorWithoutNoSource
#[derive(Debug)]
struct NotErrorWithoutNoSource;

#[derive(Error, Debug)]
#[error("...")]
pub enum E {
    Bad(#[from] NotErrorWithoutNoSource),
}

fn main() {}
