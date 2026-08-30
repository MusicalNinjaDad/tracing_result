#![cfg_attr(unstable_never_type, feature(never_type))]
#![cfg_attr(unstable_try_trait_v2, feature(try_trait_v2))]
#![cfg_attr(unstable_try_trait_v2_residual, feature(try_trait_v2_residual))]

//! A library for ergonomic error handling with tracing support.
//!
//! This crate provides [`TracingResult`], a wrapper around [`Result`] that automatically
//! emits tracing messages when errors occur. It integrates seamlessly with Rust's
//! [`Try`] trait and the `?` operator.
//!
//! # Overview
//!
//! The [`Trace`] trait extends [`Result`] with the [`and_warn`][Trace::and_warn] method,
//! which converts a [`Result`] into a [`TracingResult`] with a custom warning message.
//! When used with the `?` operator, errors will automatically log the message via [`tracing::warn`].
//!
//! # Example
//!
//! ```
//! use tracing_result::{Trace, TracingResult};
//!
//! fn might_fail() -> Result<u32, String> {
//!     Err("something went wrong".to_string())
//! }
//!
//! fn compute() -> Result<u32, String> {
//!     might_fail().and_warn("Failed to compute value")?;
//!     Ok(42)
//! }
//!
//! // When compute() is called and might_fail() returns Err,
//! // "Failed to compute value" will be logged as a warning.
//! ```
//!
//! # Features
//!
//! - Automatic tracing on error propagation via `?`
//! - Compatible with Rust's unstable `Try` trait v2 (when enabled)

use std::ops::{ControlFlow, FromResidual, Residual, Try};

/// A result type that emits tracing warnings when errors occur.
///
/// [`TracingResult`] acts like the standard [`Result`] type, adding the ability to
/// associate a custom warning message with errors. When an error is propagated
/// using the `?` operator, the message is automatically logged via [`tracing::warn`].
///
/// It is usually constructed via [Result::and_warn][Trace::and_warn] and rarely returned
/// directly - `?` will work in a block which returns [`Result`]
///
/// # Example
///
/// ```
/// use tracing_result::{Trace, TracingResult};
///
/// fn divide(a: i32, b: i32) -> Result<i32, String> {
///     if b == 0 {
///         let oops: TracingResult<_, _> = Err("division by zero".to_string())
///             .and_warn("Cannot divide by zero");
///         oops?;
///     }
///     Ok(a / b)
/// }
/// ```
pub enum TracingResult<T, E> {
    /// Success case containing the result value.
    Ok(T),
    /// Error case containing both the error and a message to be logged.
    ///
    /// The `msg` field is logged via [`tracing::warn`] when the error
    /// is propagated using the `?` operator.
    Err { err: E, msg: String },
}

impl<T, E> Try for TracingResult<T, E> {
    type Output = T;

    type Residual = TracingResult<!, E>;

    fn from_output(output: Self::Output) -> Self {
        Self::Ok(output)
    }

    /// Executes the `Try` branch operation, **logging error messages on `?``**`.
    ///
    /// When the result is [`Err`], the associated message is logged via [`tracing::warn`]
    /// and a [`ControlFlow::Break`] is returned with the error. For [`Ok`] values,
    /// [`ControlFlow::Continue`] is returned with the unwrapped value.
    ///
    /// This is the mechanism that enables automatic tracing when using the `?` operator.
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self {
            TracingResult::Ok(val) => ControlFlow::Continue(val),
            TracingResult::Err { err, msg } => {
                tracing::warn!("{msg}");
                ControlFlow::Break(TracingResult::Err { err, msg })
            }
        }
    }
}

impl<T, E> FromResidual for TracingResult<T, E> {
    fn from_residual(_residual: <Self as Try>::Residual) -> Self {
        todo!("from residual")
    }
}

impl<T, E> FromResidual<TracingResult<!, E>> for Result<T, E> {
    fn from_residual(residual: TracingResult<!, E>) -> Self {
        match residual {
            TracingResult::Err { err, .. } => Result::Err(err),
        }
    }
}

impl<T, E> Residual<T> for TracingResult<!, E> {
    type TryType = TracingResult<T, E>;
}

/// A trait for converting results into tracing results with warning messages.
///
/// This trait extends [`Result<T, E>`] with the [`and_warn`][Trace::and_warn] method,
/// which attaches a custom warning message to errors. When the resulting [`TracingResult`] is
/// used with the `?` operator, the message will be automatically logged.
///
/// # Example
///
/// ```
/// use tracing_result::Trace;
///
/// fn might_fail() -> Result<i32, String> {
///     Err("network error".to_string())
/// }
///
/// fn process() -> Result<i32, String> {
///     let result = might_fail().and_warn("Failed to fetch data");
///     // If might_fail() returns Err, "Failed to fetch data" will be logged
///     // when result is used with ?
///     result?;
///     Ok(42)
/// }
/// ```
pub trait Trace<T, E> {
    /// Attaches a warning message to this result.
    ///
    /// Converts the [`Result`] into a [`TracingResult`] that will log the
    /// provided message via [`tracing::warn`] if an error occurs and is
    /// propagated with the `?` operator.
    ///
    /// # Example
    ///
    /// ```
    /// use tracing_result::Trace;
    ///
    /// let result: Result<i32, String> = Err("io error".to_string());
    /// let tracing_result = result.and_warn("File read failed");
    ///
    /// // When tracing_result? is used, "File read failed" will be logged
    /// ```
    fn and_warn<S: ToString>(self, msg: S) -> TracingResult<T, E>;
}

impl<T, E> Trace<T, E> for Result<T, E> {
    fn and_warn<S: ToString>(self, msg: S) -> TracingResult<T, E> {
        match self {
            Ok(val) => TracingResult::Ok(val),
            Err(err) => TracingResult::Err {
                err,
                msg: msg.to_string(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use tracing_test::traced_test;

    use super::*;

    /// Tests that `and_warn` on an Ok result does not log a warning.
    #[traced_test]
    #[test]
    fn and_warn_ok() {
        fn no_error() -> Result<(), ()> {
            Ok(()).and_warn("stuff")?;
            Ok(())
        }

        assert!(no_error().is_ok());
        assert!(!logs_contain("stuff"));
    }

    /// Tests that `and_warn` on an Err result logs the warning message.
    #[traced_test]
    #[test]
    fn and_warn_err() {
        fn err() -> Result<(), ()> {
            Err(()).and_warn("stuff")?;
            Ok(())
        }

        assert!(err().is_err());
        assert!(!logs_contain("msg"));
        assert!(logs_contain("stuff"));
    }
}
