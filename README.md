# wherror

[![github]](https://github.com/dra11y/wherror)&ensp;[![crates-io]](https://crates.io/crates/wherror)&ensp;[![docs-rs]](https://docs.rs/wherror)&ensp;[![changelog]](https://github.com/dra11y/wherror/blob/main/CHANGELOG.md)

[github]: https://img.shields.io/badge/github-dra11y/wherror-8da0cb?style=for-the-badge&labelColor=555555&logo=github
[crates-io]: https://img.shields.io/crates/v/wherror.svg?style=for-the-badge&color=fc8d62&logo=rust
[docs-rs]: https://img.shields.io/badge/docs.rs-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs
[changelog]: https://img.shields.io/badge/changelog-Keep%20a%20Changelog-E05735?style=for-the-badge&labelColor=555555&logo=keepachangelog

<br>

The same `derive(Error)` macro `thiserror` provides + **the features you want!**

**wherror** = **thiserror** + **WHERE** your errors occurred 🎯

### Why Choose wherror Over thiserror?

wherror implements **the most requested community features**:

| Feature | wherror | thiserror | Community Interest |
|---------|---------|-----------|-------------------|
| **Drop-in replacement** for existing code | ✅ | ✅ | Zero migration effort |
| **Automatically use `Debug` as `Display`** with `#[error(debug)]` | ✅ | ❌ | [#172 - not planned!][thiserror#172] |
| **Call-site location tracking** | ✅ | ❌ | [#142 - 17👍 since 2021][thiserror#142] |
| **Enhanced ergonomics** (`Box<T>` unwrapping, `.location()` method) | ✅ | ❌ | wherror enhancements |

Use wherror when you need these features today, with the same reliable API you know and love.

See the [CHANGELOG](https://github.com/dra11y/wherror/blob/main/CHANGELOG.md)

[`std::error::Error`]: https://doc.rust-lang.org/std/error/trait.Error.html
[`std::panic::Location`]: https://doc.rust-lang.org/std/panic/struct.Location.html
[thiserror]: https://github.com/dtolnay/thiserror
[thiserror#142]: https://github.com/dtolnay/thiserror/issues/142
[thiserror#172]: https://github.com/dtolnay/thiserror/issues/172
[thiserror#291]: https://github.com/dtolnay/thiserror/pull/291

```toml
[dependencies]
wherror = "2"
```

### 🎯 Instant Error Location Tracking

Know **exactly where** your errors originated with zero boilerplate:

```rust
use wherror::Error;

#[derive(Error, Debug)]
#[error("Failed at {location}: {source}")]
pub struct MyError {
    #[from]
    source: std::io::Error,
    location: &'static std::panic::Location<'static>,  // ✨ Auto-populated
}

// Location automatically captured when using `?` - no manual work required!
fn read_file() -> Result<String, MyError> {
    let content = std::fs::read_to_string("file.txt")?;  // 📍 Location captured here
    Ok(content)
}
```

### 🚀 Debug Fallback - No More Boilerplate Messages

Sometimes your enum variant names *are* the error message. wherror lets you skip
the redundant `#[error("...")]` attributes that thiserror forces you to write:

```rust
use wherror::Error;

#[derive(Error, Debug)]
#[error(debug)]  // 🎉 Fallback for variants without explicit messages
pub enum ValidationError {
    #[error("Email must contain @ symbol")]  // Custom message when needed
    InvalidEmail,

    // These use Debug formatting automatically - no boilerplate! ✨
    TooShort,
    TooLong,
    EmptyInput,
    InvalidCharacters { found: char, position: usize },
}
```

<br>

## Example: All Features in Action

```rust
use wherror::Error;

#[derive(Error, Debug)]
#[error(debug)]  // ✨ Fallback for variants without explicit messages
pub enum DataStoreError {
    #[error("data store disconnected at {location}")]  // 🎯 Location tracking
    Disconnect {
        #[from]
        source: io::Error,
        location: &'static std::panic::Location<'static>,  // Auto-captured
    },
    #[error("the data for key `{0}` is not available")]
    Redaction(String),
    #[error("invalid header (expected {expected:?}, found {found:?})")]
    InvalidHeader { expected: String, found: String },
    // ✨ These use Debug formatting automatically - no #[error("...")] needed!
    Unknown,
    ConfigurationMissing,
    PermissionDenied { user_id: u64 },
}
```

<br>

## Quick Migration from thiserror

**Step 1:** Update your `Cargo.toml`:
```toml
[dependencies]
# thiserror = "2"  # Replace this
wherror = "2"       # With this
```

**Step 2:** Update imports:
```rust
// use thiserror::Error;  // Replace this
use wherror::Error;      // With this
```

**Step 3:** Your existing code works unchanged! Optionally add new features like location tracking.

<br>

## Detailed Features

wherror extends thiserror with community-requested features while maintaining
thiserror API compatibility. All existing thiserror code works unchanged.

- Errors may be enums, structs with named fields, tuple structs, or unit
  structs.

- A `Display` impl is generated for your error if you provide `#[error("...")]`
  messages on the struct or each variant of your enum, as shown above in the
  example.

  The messages support a shorthand for interpolating fields from the error.

    - `#[error("{var}")]`&ensp;⟶&ensp;`write!("{}", self.var)`
    - `#[error("{0}")]`&ensp;⟶&ensp;`write!("{}", self.0)`
    - `#[error("{var:?}")]`&ensp;⟶&ensp;`write!("{:?}", self.var)`
    - `#[error("{0:?}")]`&ensp;⟶&ensp;`write!("{:?}", self.0)`

  These shorthands can be used together with any additional format args, which
  may be arbitrary expressions. For example:

  ```rust
  # use core::i32;
  # use wherror::Error;
  #
  #[derive(Error, Debug)]
  pub enum Error {
      #[error("invalid rdo_lookahead_frames {0} (expected < {max})", max = i32::MAX)]
      InvalidLookahead(u32),
  }
  ```

  If one of the additional expression arguments needs to refer to a field of the
  struct or enum, then refer to named fields as `.var` and tuple fields as `.0`.

  ```rust
  # use wherror::Error;
  #
  # fn first_char(s: &String) -> char {
  #     s.chars().next().unwrap()
  # }
  #
  # #[derive(Debug)]
  # struct Limits {
  #     lo: usize,
  #     hi: usize,
  # }
  #
  #[derive(Error, Debug)]
  pub enum Error {
      #[error("first letter must be lowercase but was {:?}", first_char(.0))]
      WrongCase(String),
      #[error("invalid index {idx}, expected at least {} and at most {}", .limits.lo, .limits.hi)]
      OutOfBounds { idx: usize, limits: Limits },
  }
  ```

- A `From` impl is generated for each variant that contains a `#[from]`
  attribute.

  The variant using `#[from]` must not contain any other fields beyond the
  source error (and possibly a location or backtrace &mdash; see below). Usually `#[from]`
  fields are unnamed, but `#[from]` is allowed on a named field too.

  ```rust
  # use core::fmt::{self, Display};
  # use std::io;
  # use wherror::Error;
  #
  # mod globset {
  #     #[derive(wherror::Error, Debug)]
  #     #[error("...")]
  #     pub struct Error;
  # }
  #
  #[derive(Error, Debug)]
  pub enum MyError {
      Io(#[from] io::Error),
      Glob(#[from] globset::Error),
  }
  #
  # impl Display for MyError {
  #     fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
  #         unimplemented!()
  #     }
  # }
  ```

  For `Box<T>` fields with `#[from]`, both `From<Box<T>>` and `From<T>`
  implementations are automatically generated for enhanced ergonomics:

  ```rust
  # use std::io;
  # use wherror::Error;
  #
  #[derive(Error, Debug)]
  #[error("Error occurred")]
  pub struct MyError {
      #[from]
      source: Box<io::Error>,
  }
  #
  # fn example() -> Result<(), Box<dyn std::error::Error>> {
  #     let io_error = io::Error::new(io::ErrorKind::Other, "test");
  #
  #     // Both work:
  #     let err1: MyError = Box::new(io_error).into();
  #     let err2: MyError = io::Error::new(io::ErrorKind::Other, "test").into();  // automatically boxed
  #     Ok(())
  # }
  ```

- Use `#[error(debug)]` as a fallback to automatically generate Display
  implementations using the Debug format. This eliminates boilerplate when your
  enum variant names are already descriptive error messages.

  This addresses the request in [thiserror#172] for optional error messages,
  allowing you to skip redundant `#[error("TooSmall")]` when `TooSmall` is
  already a clear error name.

  For enums, you can apply `#[error(debug)]` at the type level to automatically
  generate Display for all variants that don't have explicit `#[error("...")]`
  messages:

  ```rust
  # use wherror::Error;
  #
  #[derive(Error, Debug)]
  #[error(debug)]  // fallback for variants without explicit messages
  pub enum MyError {
      #[error("Custom message: {0}")]
      WithMessage(String),

      // These will use Debug formatting:
      Simple,
      Complex { code: u32, message: String },
      WithData(i32, String),
  }
  ```

  You can also apply `#[error(debug)]` to individual variants or struct types:

  ```rust
  # use wherror::Error;
  #
  #[derive(Error, Debug)]
  pub enum MyError {
      #[error("IO error: {0}")]
      Io(std::io::Error),

      #[error(debug)]  // This variant uses Debug formatting
      Other { details: String, code: i32 },
  }

  #[derive(Error, Debug)]
  #[error(debug)]  // Entire struct uses Debug formatting
  pub struct DebugError {
      message: String,
      code: u32,
  }
  ```

- The Error trait's `source()` method is implemented to return whichever field
  has a `#[source]` attribute or is named `source`, if any. This is for
  identifying the underlying lower level error that caused your error.

  The `#[from]` attribute always implies that the same field is `#[source]`, so
  you don't ever need to specify both attributes.

  Any error type that implements `std::error::Error` or dereferences to `dyn
  std::error::Error` will work as a source.

  ```rust
  # use core::fmt::{self, Display};
  # use wherror::Error;
  #
  #[derive(Error, Debug)]
  pub struct MyError {
      msg: String,
      #[source]  // optional if field name is `source`
      source: anyhow::Error,
  }
  #
  # impl Display for MyError {
  #     fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
  #         unimplemented!()
  #     }
  # }
  ```

- Fields of type `&'static std::panic::Location<'static>` are automatically
  populated with the call site location when errors are created via `From` trait
  conversion. This works seamlessly with the `?` operator for precise error tracking.

  wherror automatically generates a `.location()` method that returns
  `Option<&'static std::panic::Location<'static>>` for easy access to error origins.

  This implements the feature requested in [thiserror#142] (17👍).

  ```rust
  # use wherror::Error;
  #
  #[derive(Error, Debug)]
  #[error("Parse error at {location}: {source}")]
  pub struct ParseError {
      #[from]
      source: std::num::ParseIntError,
      location: &'static std::panic::Location<'static>,  // automatically detected
  }

  fn example() -> Result<(), ParseError> {
      let _number: i32 = "not_a_number".parse()?;  // Location captured here automatically
      Ok(())
  }

  # fn demonstrate_usage() {
  #     if let Err(error) = example() {
  #         // Access the location where the error occurred
  #         if let Some(location) = error.location() {
  #             eprintln!("Error occurred at {}:{}", location.file(), location.line());
  #         }
  #     }
  # }
  ```

- The Error trait's `provide()` method is implemented to provide whichever field
  has a type named `Backtrace`, if any, as a `std::backtrace::Backtrace`. Using
  `Backtrace` in errors requires a nightly compiler with Rust version 1.73 or
  newer.

  ```rust,ignore
  # use std::backtrace::Backtrace;
  # use wherror::Error;
  #
  #[derive(Error, Debug)]
  #[error("Something went wrong: {msg}")]
  pub struct MyError {
      msg: String,
      backtrace: Backtrace,  // automatically detected
  }
  ```

- If a field is both a source (named `source`, or has `#[source]` or `#[from]`
  attribute) *and* is marked `#[backtrace]`, then the Error trait's `provide()`
  method is forwarded to the source's `provide` so that both layers of the error
  share the same backtrace. The `#[backtrace]` attribute requires a nightly
  compiler with Rust version 1.73 or newer.

  ```rust,ignore
  # use std::io;
  # use wherror::Error;
  #
  #[derive(Error, Debug)]
  #[error("IO error occurred")]
  pub enum MyError {
      Io {
          #[backtrace]
          source: io::Error,
      },
  }
  ```

- For variants that use `#[from]` and also contain a `Backtrace` field, a
  backtrace is captured from within the `From` impl.

  ```rust,ignore
  # use std::backtrace::Backtrace;
  # use std::io;
  # use wherror::Error;
  #
  #[derive(Error, Debug)]
  #[error("IO error occurred")]
  pub enum MyError {
      Io {
          #[from]
          source: io::Error,
          backtrace: Backtrace,
      },
  }
  ```

- Errors may use `error(transparent)` to forward the source and Display methods
  straight through to an underlying error without adding an additional message.
  This would be appropriate for enums that need an "anything else" variant.

  ```
  # use wherror::Error;
  #
  #[derive(Error, Debug)]
  pub enum MyError {
      # /*
      ...
      # */

      #[error(transparent)]
      Other(#[from] anyhow::Error),  // source and Display delegate to anyhow::Error
  }
  ```

  Another use case is hiding implementation details of an error representation
  behind an opaque error type, so that the representation is able to evolve
  without breaking the crate's public API.

  ```
  # use wherror::Error;
  #
  // PublicError is public, but opaque and easy to keep compatible.
  #[derive(Error, Debug)]
  #[error(transparent)]
  pub struct PublicError(#[from] ErrorRepr);

  impl PublicError {
      // Accessors for anything we do want to expose publicly.
  }

  // Private and free to change across minor version of the crate.
  #[derive(Error, Debug)]
  enum ErrorRepr {
      # /*
      ...
      # */
  }
  ```

- See also the [`anyhow`] library for a convenient single error type to use in
  application code.

  [`anyhow`]: https://github.com/dtolnay/anyhow

<br>

### Comparison to anyhow

Use wherror if you care about designing your own dedicated error type(s) so
that the caller receives exactly the information that you choose in the event of
failure. This most often applies to library-like code. Use [Anyhow] if you don't
care what error type your functions return, you just want it to be easy. This is
common in application-like code.

[Anyhow]: https://github.com/dtolnay/anyhow

<br>

#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>

<br>

#### Attribution

<sup>
Fork of <a href="https://github.com/dra11y/wherror">thiserror</a> by David Tolnay,
with location support by <a href="https://github.com/onlycs">Angad Tendulkar</a>
from <a href="https://github.com/dtolnay/thiserror/pull/291">thiserror#291</a>.
</sup>

License: MIT OR Apache-2.0
