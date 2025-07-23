// use std::fmt;
// use wherror::Error;

// // A type that doesn't implement Debug
// struct NoDebug {
//     value: i32,
// }

// // Test case 1: variant-level #[error(debug)] - should fail
// #[derive(Error)]
// enum MyError {
//     #[error(debug)]
//     Variant2(NoDebug), // This field doesn't implement Debug
// }

// // Test case 2: enum-level #[error(debug)] - should fail
// #[derive(Error)]
// #[error(debug)]
// enum MyErrorEnum {
//     Variant2(NoDebug), // This field doesn't implement Debug
// }

// // Manual Debug implementation for MyError
// impl fmt::Debug for MyError {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         match self {
//             MyError::Variant2(field) => f
//                 .debug_tuple("Variant2")
//                 .field(&format!("NoDebug({})", field.value))
//                 .finish(),
//         }
//     }
// }

// // Manual Debug implementation for MyErrorEnum
// impl fmt::Debug for MyErrorEnum {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         match self {
//             MyErrorEnum::Variant2(field) => f
//                 .debug_tuple("Variant2")
//                 .field(&format!("NoDebug({})", field.value))
//                 .finish(),
//         }
//     }
// }

// fn main() {
//     let error2 = MyError::Variant2(NoDebug { value: 123 });
//     println!("Error2: {}", error2);
// }
