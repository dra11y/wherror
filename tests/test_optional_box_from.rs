use std::io;
use wherror::Error;

#[derive(Error, Debug)]
#[error("optional box error")]
pub struct OptionalBoxError {
    #[from]
    source: Option<Box<io::Error>>, // Should generate From<Option<Box<io::Error>>> and From<io::Error>
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optional_box_from() {
        let io_error = io::Error::new(io::ErrorKind::Other, "test error");
        let boxed_error = Box::new(io_error);
        let error = OptionalBoxError::from(boxed_error);
        println!("Created error from Box<...>: {error:?}");
    }

    #[test]
    fn test_optional_unboxed_from() {
        let io_error = io::Error::new(io::ErrorKind::Other, "test error");
        let error = OptionalBoxError::from(io_error);
        println!("Created error from unboxed (optional): {error:?}");
    }
}
