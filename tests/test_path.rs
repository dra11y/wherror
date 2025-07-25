#![cfg(feature = "std")]

use core::fmt::Display;
use ref_cast::RefCast;
use std::path::{Path, PathBuf};
use wherror::Error;

#[derive(Error, Debug)]
#[error("failed to read '{file}'")]
struct StructPathBuf {
    file: PathBuf,
}

#[derive(Error, Debug, RefCast)]
#[repr(C)]
#[error("failed to read '{file}'")]
struct StructPath {
    file: Path,
}

#[derive(Error, Debug)]
enum EnumPathBuf {
    #[error("failed to read '{0}'")]
    Read(PathBuf),
}

#[derive(Error, Debug)]
#[error("{tail}")]
pub struct UnsizedError {
    pub head: i32,
    pub tail: str,
}

#[derive(Error, Debug)]
pub enum BothError {
    #[error("display:{0} debug:{0:?}")]
    DisplayDebug(PathBuf),
    #[error("debug:{0:?} display:{0}")]
    DebugDisplay(PathBuf),
}

fn assert<T: Display>(expected: &str, value: T) {
    assert_eq!(expected, value.to_string());
}

#[test]
fn test_display() {
    let path = Path::new("/thiserror");
    let file = path.to_owned();
    assert("failed to read '/thiserror'", StructPathBuf { file });
    let file = path.to_owned();
    assert("failed to read '/thiserror'", EnumPathBuf::Read(file));
    assert("failed to read '/thiserror'", StructPath::ref_cast(path));
}
