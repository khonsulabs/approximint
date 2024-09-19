# Approximint

<!-- This file is generated by `rustme`. Ensure you're editing the source in the .rustme/ directory --!>
<!-- markdownlint-disable first-line-h1 -->

![Approximint is considered alpha and unsupported](https://img.shields.io/badge/status-alpha-orange)
[![crate version](https://img.shields.io/crates/v/approximint.svg)](https://crates.io/crates/approximint)
[![Documentation for `main`](https://img.shields.io/badge/docs-main-informational)](https://cushy.rs/main/docs/cushy/)

A large integer library supporting a non-inclusive range of
(-1e4,294,967,305..+1e4,294,967,305) with 9 decimal digits of precision.

This library was designed to be a simple, efficient implementation of large
numbers for use in incremental games. It utilizes two 32 bit integers to
represent a number in the form of `coefficient * 10^exponent`.

## Basic usage

```rust
use approximint::Approximint;

let googol = Approximint::one_e(100);
let billion = Approximint::new(1_000_000_000);

assert_eq!((googol * billion).to_string(), "1.000e109");
assert_eq!((googol * billion).as_english().to_string(), "1 billion googol");
```

## no_std

This crate supports all integer operations, including formatting, in no_std
without alloc. Floating point operations require the `std` feature to be
enabled.

## Open-source Licenses

This project, like all projects from [Khonsu Labs](https://khonsulabs.com/), is open-source.
This repository is available under the [MIT License](./LICENSE-MIT) or the
[Apache License 2.0](./LICENSE-APACHE).

To learn more about contributing, please see [CONTRIBUTING.md](./CONTRIBUTING.md).
