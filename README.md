# Tracing Result

A library for ergonomic error handling with tracing support.

This crate provides a wrapper around [`Result`], [`Option`] & any other types
which implement [`Try`] that automatically emits tracing messages at configurable log levels
when unpacked via the `?` operator.

## Overview

The [`Trace`] trait extends all `Try`-types `T` with methods that wrap it in a [`Traced<T>`]
with a custom tracing message. The message is emitted automatically when the [`Traced<T>`] is
used with the `?` operator.

Simply `use tracing_result::Trace` to bring the methods into scope. The blanket implementation
ensures availability on all `Try`-types, not only `Result`& `Option`.

There are two families of methods:

- **`or_*` methods** ([`or_warn`][Trace::or_warn], [`or_error`][Trace::or_error],
  [`or_debug`][Trace::or_debug], [`or_trace`][Trace::or_trace]): Log the message when an
  **`Err`**/**`None`** etc. is unpacked
- **`and_*` methods** ([`and_warn`][Trace::and_warn], [`and_error`][Trace::and_error],
  [`and_debug`][Trace::and_debug], [`and_trace`][Trace::and_debug]):
  Log the message when an **`Ok`**/**`Some`** etc. is unpacked

## Examples

### `Traced<Result>`

```rust
use std::io;
use tracing_result::Trace;

fn might_fail() -> io::Result<u32> {
    Err(io::Error::other("something went wrong"))
}

fn compute() -> io::Result<u32> {
    // If might_fail() returns Err, "Failed to compute value" will be logged as a warning
    might_fail().or_warn("Failed to compute value")?;
    Ok(42)
}

fn unexpected_success() -> io::Result<u32> {
    // If Ok is returned, "Unexpected: computation succeeded" will be logged as a warning
    let res = io::Result::Ok(42).and_warn("Unexpected: computation succeeded")?;
    Ok(res)
}
```

### `Traced<Option>`

```rust
use tracing_result::Trace;
use try_v2::{Extract, Transform};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Data {
    value: Option<i32>,
}

let data: Vec<Data> = Vec::new();

let first_value = data
    .first()
    .or_warn("No entries found") // Emitted if `None`
    .and_then(|d| d.value) // `Transform` methods call `Try::branch` and therefore emit
    .or_warn("No value in first entry") // Emitted if `None`
    .unwrap_or_default(); // `Extract` methods call `Try::branch` and therefore emit

assert_eq!(first_value, 0);
```
