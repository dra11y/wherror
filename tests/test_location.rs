use std::{fmt::Debug, io, panic::Location};
use wherror::Error;

#[derive(Error, Debug)]
enum MError {
    #[error("At {location}: location test error, sourced from {other}")]
    Test {
        #[from]
        other: io::Error,
        location: &'static Location<'static>,
    },
}

#[derive(Error, Debug)]
enum TupleError {
    #[error("Tuple variant at {1}: sourced from {0}")]
    Test(#[from] io::Error, &'static Location<'static>),
}

#[derive(Error, Debug)]
#[error("At {location} test error, sourced from {other}")]
pub struct TestError {
    #[from]
    other: io::Error,
    location: &'static Location<'static>,
}

#[test]
#[should_panic]
fn test_enum() {
    fn inner() -> Result<(), MError> {
        Err(io::Error::new(io::ErrorKind::AddrInUse, String::new()))?;
        Ok(())
    }

    inner().unwrap();
}

#[test]
#[should_panic]
fn test_tuple_enum() {
    fn inner() -> Result<(), TupleError> {
        Err(io::Error::new(io::ErrorKind::AddrInUse, String::new()))?;
        Ok(())
    }

    inner().unwrap();
}

#[test]
#[should_panic]
fn test_struct() {
    fn inner() -> Result<(), TestError> {
        Err(io::Error::new(io::ErrorKind::AddrInUse, String::new()))?;
        Ok(())
    }

    inner().unwrap();
}

#[test]
fn test_location_method() {
    // Test struct with location method
    let error: TestError = io::Error::new(io::ErrorKind::AddrInUse, String::new()).into();
    assert!(error.location().is_some());

    // Test enum with location method
    let error: MError = io::Error::new(io::ErrorKind::AddrInUse, String::new()).into();
    assert!(error.location().is_some());

    // Test tuple enum with location method
    let error: TupleError = io::Error::new(io::ErrorKind::AddrInUse, String::new()).into();
    assert!(error.location().is_some());
}

// Error types without location fields - these won't have .location() method at all
#[derive(Error, Debug)]
#[error("No location error")]
pub struct NoLocationError {
    #[from]
    other: io::Error,
}

// Test enum with mixed variants - some with location, some without
#[derive(Error, Debug)]
enum MixedLocationEnum {
    #[error("With location: {location}")]
    WithLocation {
        #[from]
        other: io::Error,
        location: &'static Location<'static>,
    },
    #[error("Without location")]
    WithoutLocation { message: String },
}

#[test]
fn test_location_method_mixed_enum() {
    // Test variant with location field returns Some
    let error: MixedLocationEnum = io::Error::new(io::ErrorKind::AddrInUse, String::new()).into();
    assert!(error.location().is_some());

    // Test variant without location field returns None
    let error = MixedLocationEnum::WithoutLocation {
        message: "test".to_string(),
    };
    assert!(error.location().is_none());
}

#[test]
fn test_location_values() {
    // Test direct error creation - should capture this exact line
    let line_direct = line!();
    let error1: TestError = io::Error::new(io::ErrorKind::AddrInUse, String::new()).into();

    let location1 = error1.location().expect("Error should have location");
    assert_eq!(location1.file(), file!());
    assert_eq!(location1.line(), line_direct + 1);

    // Test helper WITHOUT #[track_caller] - should capture line inside helper
    let error2: TestError = create_error_without_track_caller();
    let location2 = error2.location().expect("Error should have location");
    assert_eq!(location2.file(), file!());
    // This should be different - it's the line inside the helper function
    assert_ne!(location2.line(), line_direct + 1);

    // Test helper WITH #[track_caller] - should capture the call site (this line)
    let line_with_track_caller = line!();
    let error3: TestError = create_error_helper();

    let location3 = error3.location().expect("Error should have location");
    assert_eq!(location3.file(), file!());
    // With #[track_caller], should capture where we called the helper
    assert_eq!(location3.line(), line_with_track_caller + 1);

    // Verify that direct creation and #[track_caller] helper give different locations
    assert_ne!(location1.line(), location3.line());

    // Verify that non-track_caller helper gives different location than both
    assert_ne!(location2.line(), location1.line());
    assert_ne!(location2.line(), location3.line());
}

// Helper function WITH #[track_caller] - should capture caller's location
// This is the recommended pattern for error creation helpers
#[track_caller]
fn create_error_helper() -> TestError {
    io::Error::new(io::ErrorKind::AddrInUse, String::new()).into()
}

// Helper function WITHOUT #[track_caller] - captures location inside function
fn create_error_without_track_caller() -> TestError {
    io::Error::new(io::ErrorKind::AddrInUse, String::new()).into()
}
