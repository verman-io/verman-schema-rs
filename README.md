verman-schema-rs
================
[![License](https://img.shields.io/badge/license-Apache--2.0%20OR%20MIT-blue.svg)](https://opensource.org/licenses/Apache-2.0)

Schema for verMan version managers. Currently this does—or in short order; shall—include JSON | TOML examples and serde Rust `struct`s.

For more information see https://verMan.io

## Predefined constants

These constants are accessible in the TOML/YAML/JSON file using `${}` syntax, like `${OS} == \"windows\"` in a `when` field.

- `ARCH`, string (see https://github.com/rust-lang/rust/blob/1.77.0/library/std/src/env.rs#L916-L936)
- `FAMILY`, string (see https://github.com/rust-lang/rust/blob/1.77.0/library/std/src/env.rs#L938-L945)
- `OS`, string (see https://github.com/rust-lang/rust/blob/1.77.0/library/std/src/env.rs#L947-L961)
- `BUILD_TIME`, `std::time::SystemTime`

<hr/>

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
