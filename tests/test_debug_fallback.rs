use core::fmt::Display;
use wherror::Error;

fn assert<T: Display>(expected: &str, value: T) {
    assert_eq!(expected, value.to_string());
}

#[test]
fn test_debug_fallback() {
    #[derive(Error, Debug)]
    #[error(debug)]
    enum MyError {
        #[error("Something went wrong: {0}")]
        WithMessage(String),

        TooSmall,
        TooBig,
        InvalidValue(i32),
        ComplexVariant {
            expected: String,
            found: String,
        },
    }

    // Test unit variants
    assert("TooSmall", MyError::TooSmall);
    assert("TooBig", MyError::TooBig);

    // Test tuple variant
    assert("InvalidValue(42)", MyError::InvalidValue(42));

    // Test struct variant
    assert(
        "ComplexVariant { ... }",
        MyError::ComplexVariant {
            expected: "foo".to_string(),
            found: "bar".to_string(),
        },
    );

    // Test that explicit error messages still work
    assert(
        "Something went wrong: test",
        MyError::WithMessage("test".to_string()),
    );
}

#[test]
fn test_no_error_annotations() {
    #[derive(Error, Debug)]
    #[error(debug)]
    enum PureDebugError {
        Simple,
        WithData(u32),
        Complex { code: i32, message: String },
    }

    assert("Simple", PureDebugError::Simple);
    assert("WithData(123)", PureDebugError::WithData(123));
    assert(
        "Complex { ... }",
        PureDebugError::Complex {
            code: 404,
            message: "Not found".to_string(),
        },
    );
}

#[test]
fn test_mixed_error_and_debug_variants() {
    #[derive(Error, Debug)]
    #[error(debug)]
    enum MixedError {
        #[error("Detailed error: {message}")]
        Detailed {
            message: String,
        },

        Simple,
        WithData(u32),
    }

    assert(
        "Detailed error: oops",
        MixedError::Detailed {
            message: "oops".to_string(),
        },
    );
    assert("Simple", MixedError::Simple);
    assert("WithData(123)", MixedError::WithData(123));
}

#[test]
fn test_variant_level_debug() {
    #[derive(Error, Debug)]
    enum VariantError {
        #[error("Custom: {0}")]
        WithMessage(String),

        #[error(debug)]
        Simple,

        #[error(debug)]
        WithData(u32),

        #[error(debug)]
        Complex { code: i32, message: String },
    }

    assert(
        "Custom: test",
        VariantError::WithMessage("test".to_string()),
    );
    assert("Simple", VariantError::Simple);
    assert("WithData(123)", VariantError::WithData(123));
    assert(
        "Complex { code: 404, message: \"Not found\" }",
        VariantError::Complex {
            code: 404,
            message: "Not found".to_string(),
        },
    );
}

#[test]
fn test_mixed_auto_and_variant_debug() {
    #[derive(Error, Debug)]
    #[error(debug)]
    enum MixedAutoError {
        #[error("Detailed error: {message}")]
        Detailed {
            message: String,
        },

        #[error(debug)]
        ExplicitDebug(i32),

        Simple,
        WithData(u32),
    }

    assert(
        "Detailed error: oops",
        MixedAutoError::Detailed {
            message: "oops".to_string(),
        },
    );
    assert("ExplicitDebug(42)", MixedAutoError::ExplicitDebug(42));
    assert("Simple", MixedAutoError::Simple);
    assert("WithData(123)", MixedAutoError::WithData(123));
}

#[test]
fn test_struct_with_error_debug() {
    #[derive(Error, Debug)]
    #[error(debug)]
    #[allow(unused)]
    struct MyError {
        code: i32,
        message: String,
    }

    let error = MyError {
        code: 404,
        message: "Not found".to_string(),
    };

    assert_eq!(
        "MyError { code: 404, message: \"Not found\" }",
        error.to_string()
    );
}

#[test]
fn test_struct_with_different_field_types() {
    #[derive(Error, Debug)]
    #[error(debug)]
    #[allow(unused)]
    struct ComplexError {
        id: u64,
        active: bool,
        reasons: Vec<String>,
    }

    let error = ComplexError {
        id: 12345,
        active: false,
        reasons: vec!["timeout".to_string(), "retry failed".to_string()],
    };

    let result = error.to_string();
    assert!(result.contains("ComplexError"));
    assert!(result.contains("id: 12345"));
    assert!(result.contains("active: false"));
    assert!(result.contains("timeout"));
    assert!(result.contains("retry failed"));
}

#[test]
fn test_struct_unit_like() {
    #[derive(Error, Debug)]
    #[error(debug)]
    struct UnitError;

    let error = UnitError;
    assert_eq!("UnitError", error.to_string());
}
