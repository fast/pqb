```text
                   __
    ____   ____ _ / /_
   / __ \ / __ `// __ \
  / /_/ // /_/ // /_/ /
 / .___/ \__, //_.___/
/_/        /_/
```

[![Crates.io][crates-badge]][crates-url]
[![Documentation][docs-badge]][docs-url]
[![MSRV 1.93.0][msrv-badge]](https://www.whatrustisit.com)
[![Apache 2.0 licensed][license-badge]][license-url]
[![Build Status][actions-badge]][actions-url]

[crates-badge]: https://img.shields.io/crates/v/pqb.svg
[crates-url]: https://crates.io/crates/pqb
[docs-badge]: https://img.shields.io/docsrs/pqb
[docs-url]: https://docs.rs/pqb
[msrv-badge]: https://img.shields.io/badge/MSRV-1.93.0-green?logo=rust
[license-badge]: https://img.shields.io/crates/l/pqb
[license-url]: LICENSE
[actions-badge]: https://github.com/tisonkun/pqb/workflows/CI/badge.svg
[actions-url]: https://github.com/tisonkun/pqb/actions?query=workflow%3ACI

`pqb` is a PostgreSQL query builder.

## Installation

```shell
cargo add pqb
```

## Minimum Rust version policy

This crate is built against the latest stable release, and its minimum supported rustc version is 1.93.0.

The policy is that the minimum Rust version required to use this crate can be increased in minor version updates. For example, if version 1.0 requires Rust 1.60.0, then version 1.0.z for all values of z will also require Rust 1.60.0 or newer. However, version 1.y for y > 0 may require a newer minimum version of Rust.

## License

This project is licensed under [Apache License, Version 2.0](LICENSE).

## Origins

This project is derived from [sea-query](https://github.com/SeaQL/sea-query) with significant simplification since it is aimed at generating PostgreSQL only statements (queries).

We don't need to abstract over multiple database backends, so many traits and tricks in sea-query are removed to reduce complexity.
