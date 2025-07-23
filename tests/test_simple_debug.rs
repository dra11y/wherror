use wherror::Error;

// Test if error(debug) works at the struct level
#[derive(Error, Debug)]
#[error(debug)]
struct SimpleError {
    msg: String,
}
