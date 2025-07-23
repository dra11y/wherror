use std::error::Error as StdError;
use std::io;
use wherror::Error;

#[derive(Error, Debug)]
pub enum TestEnumFromBox {
    #[error("io error")]
    Io {
        #[from]
        source: Box<io::Error>,
    },
    #[error("other error")]
    Other,
}

#[test]
fn test_enum_box_from() {
    // This should work with the current From<Box<io::Error>> impl
    let io_error = io::Error::new(io::ErrorKind::Other, "test error");
    let boxed_error = Box::new(io_error);
    let error = TestEnumFromBox::from(boxed_error);
    println!("Created enum error from Box: {:?}", error);
}

#[test]
fn test_enum_unboxed_from() {
    // This should now work with our new From<io::Error> impl
    let io_error = io::Error::new(io::ErrorKind::Other, "test error");
    let error = TestEnumFromBox::from(io_error); // This should work now!
    println!("Created enum error from unboxed: {:?}", error);

    // Test that the error was properly boxed
    assert!(error.source().is_some());
}
