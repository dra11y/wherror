use std::error::Error as StdError;
use std::io;
use wherror::Error;

#[derive(Error, Debug)]
#[error("test from box")]
pub struct TestFromBox {
    #[from]
    source: Box<io::Error>,
}

#[test]
fn test_box_from() {
    let io_error = io::Error::new(io::ErrorKind::Other, "test error");
    let boxed_error = Box::new(io_error);
    let error = TestFromBox::from(boxed_error);
    println!("Created error from Box: {error:?}");
}

#[test]
fn test_unboxed_from() {
    let io_error = io::Error::new(io::ErrorKind::Other, "test error");
    let error = TestFromBox::from(io_error); // This should work now!
    println!("Created error from unboxed: {error:?}");
    assert!(error.source().is_some());
}
