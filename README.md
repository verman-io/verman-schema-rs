verman-schema-rs
================
[![License](https://img.shields.io/badge/license-Apache--2.0%20OR%20MIT-blue.svg)](https://opensource.org/licenses/Apache-2.0)

Schema for verMan version managers. Currently, this includes JSON & TOML examples and serde Rust `struct`s.

For more information see https://verMan.io

## Predefined constants

These constants are accessible in the TOML/YAML/JSON file using `${}` syntax, like `${OS} == \"windows\"` in a `when` field.

  - `ARCH`, string (see https://github.com/rust-lang/rust/blob/1.77.0/library/std/src/env.rs#L916-L936)
  - `FAMILY`, string (see https://github.com/rust-lang/rust/blob/1.77.0/library/std/src/env.rs#L938-L945)
  - `OS`, string (see https://github.com/rust-lang/rust/blob/1.77.0/library/std/src/env.rs#L947-L961)
  - `BUILD_TIME`, `std::time::SystemTime`

## Resolving configuration

Configuration resolution should be straightforward. To remove ambiguity, this is documented below.

  0. System environment variables added to internal dictionary `vars`
  1. `constants` upserted to internal dictionary `vars`
  2. `env_vars` JSON-objects upserted to internal dictionary `vars`
  3. `String`s `${NAME}` evaluated using aforementioned `vars` and predefined constants; when `NAME` not found `${NAME}` is left as `${NAME}`
  4. `String`s with shebang evaluation
    a) implicitly takes config file-contents as `stdin`
    b) aforementioned `vars` are made available to shebang-evaluated
    c) `#!/jq` isn't real [`jq`](https://jqlang.github.io/jq) but the `#RewriteInRust` [`jaq`](https://github.com/01mf02/jaq) compiled into this library
    d) Similarly, the normal shebang isn't real. This library handles execution; by reading the first line; making this far more portable (e.g., to Windows [both CMD and PowerShell]).
  5. Similar to `$ref` of [JSON-reference](https://niem.github.io/json/reference/json-schema/references) (common in [JSON-schema](https://json-schema.org/specification)) cross-referencing can occur and thus multiple passes may be required to fully-resolve variables

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
