use wherror::Error;

// Test what happens when we use #[error(debug)] without Debug derive
#[derive(Error)]
#[error(debug)]
struct TestError {
    msg: String,
}

fn main() {
    println!("Compiles!");
}
