#![cfg_attr(unstable_never_type, feature(never_type))]
#![cfg_attr(unstable_try_trait_v2, feature(try_trait_v2))]

//! A library for ergonomic error handling with tracing support.
//!
//! This crate provides [`TracingResult`], a wrapper around [`Result`] that automatically
//! emits tracing messages at configurable log levels when results are unpacked via the `?` operator.
//! It integrates seamlessly with Rust's [`Try`] trait.
//!
//! # Overview
//!
//! The [`Trace`] trait extends [`Result<T, E>`] with methods that convert a [`Result`] into a [`TracingResult`]
//! with a custom message. The message is logged automatically when the [`TracingResult`] is used with
//! the `?` operator.
//!
//! There are two families of methods:
//! - **`or_*` methods** (`or_warn`, `or_error`, `or_debug`, `or_trace`): Log the message when an **Err** is unpacked
//! - **`and_*` methods** (`and_warn`, `and_error`, `and_debug`, `and_trace`): Log the message when an **Ok** is unpacked
//!
//! # Example
//!
//! ```
//! use std::io;
//! use tracing_result::Trace;
//!
//! fn might_fail() -> io::Result<u32> {
//!     Err(io::Error::other("something went wrong"))
//! }
//!
//! fn compute() -> io::Result<u32> {
//!     // If might_fail() returns Err, "Failed to compute value" will be logged as a warning
//!     might_fail().or_warn("Failed to compute value")?;
//!     Ok(42)
//! }
//!
//! fn unexpected_success() -> io::Result<u32> {
//!     // If Ok is returned, "Unexpected: computation succeeded" will be logged as a warning
//!     let res = io::Result::Ok(42).and_warn("Unexpected: computation succeeded")?;
//!     Ok(res)
//! }
//! ```
//!
//! # Features
//!
//! - Automatic tracing on result propagation via `?`
//! - Support for all [`tracing`] log levels: ERROR, WARN, INFO, DEBUG, TRACE
//! - Compatible with Rust's unstable `Try` trait v2 (when enabled)

use std::{
    error::Error,
    ops::{ControlFlow, FromResidual, Try},
};
use tracing::Level;
use try_v2::{Extract, Transform};

use crate::TracingConfig::{OnOutput, OnResidual};

/// Configuration for tracing log level and message.
#[derive(Debug, Clone, Copy)]
pub struct Event {
    /// The tracing level to use when logging.
    pub level: Level,
    /// The message to log.
    pub message: &'static str,
}

impl Event {
    fn emit(&self) {
        match self.level {
            Level::ERROR => tracing::error!("{}", self.message),
            Level::WARN => tracing::warn!("{}", self.message),
            Level::INFO => tracing::info!("{}", self.message),
            Level::DEBUG => tracing::debug!("{}", self.message),
            Level::TRACE => tracing::trace!("{}", self.message),
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum TracingConfig {
    OnOutput(Event),
    OnResidual(Event),
    #[default]
    None,
}

impl TracingConfig {
    fn emit_output(&self) {
        if let OnOutput(event) = self {
            event.emit();
        }
    }

    fn emit_residual(&self) {
        if let OnResidual(event) = self {
            event.emit();
        }
    }
}

/// A result type that emits tracing messages at configurable log levels.
///
/// [`TracingResult`] acts like the standard [`Result`] type, adding the ability to
/// associate a custom message that is logged when the result is unpacked via the `?` operator.
/// The log level (ERROR, WARN, INFO, DEBUG, or TRACE) is determined by which [`Trace`] trait
/// method was used to construct it.
///
/// It is usually constructed via methods from the [`Trace`] trait and rarely returned
/// directly - `?` will work in a block which returns [`Result`].
///
/// # Example
///
/// ```
/// use std::io;
/// use tracing_result::{Trace, TracingResult};
///
/// fn divide(a: i32, b: i32) -> io::Result<i32> {
///     if b == 0 {
///         let oops: TracingResult<_, _> = Err(io::Error::other("division by zero"))
///             .or_warn("Cannot divide by zero");
///         oops?;
///     }
///     Ok(a / b)
/// }
/// ```
pub type TracingResult<T, E> = Traced<Result<T, E>>;

pub struct Traced<T>
where
    T: Try,
{
    inner: T,
    event: TracingConfig,
}

impl<T: Try> Try for Traced<T> {
    type Output = T::Output;

    type Residual = T::Residual;

    fn from_output(output: Self::Output) -> Self {
        Self {
            inner: Try::from_output(output),
            event: Default::default(),
        }
    }

    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self.inner.branch() {
            ControlFlow::Continue(output) => {
                self.event.emit_output();
                ControlFlow::Continue(output)
            }
            ControlFlow::Break(residual) => {
                self.event.emit_residual();
                ControlFlow::Break(residual)
            }
        }
    }
}

impl<T: Try> FromResidual for Traced<T> {
    fn from_residual(residual: <Self as Try>::Residual) -> Self {
        Self {
            inner: FromResidual::from_residual(residual),
            event: Default::default(),
        }
    }
}

/// Calling any of the extract functions *will emit* the enclosed tracing entries related
/// to the relevant variant.
///
/// E.g.
/// - `.unwrap()` on a `Traced<Result<_,_>>` will emit any entries stored on the `Ok` variant
///   but *not* any entries stored on the `Err` variant.
impl<T, O> Extract<O> for Traced<T> where T: Try<Output = O> {}

impl<T, O> Transform<O> for Traced<T> where T: Try<Output = O> {}

impl<T, E: Error> TracingResult<T, E> {
    /// Converts from TracingResult<T, E> to Option<T>, emitting any tracing entry stored
    /// in `Ok` variant (via `and_...()`) the process.
    ///
    /// Converts self into an Option<T>, consuming self, and converting the error to None, if any.
    pub fn ok(self) -> Option<T> {
        self.output()
    }

    /// Converts from TracingResult<T, E> to Option<E>, emitting any tracing  entry stored
    /// in `Err` variant (via `or_...()`) the process.
    ///
    /// Converts self into an Option<E>, consuming self, and discarding the success value, if any.
    pub fn err(self) -> Option<E> {
        match self.branch() {
            ControlFlow::Continue(_) => None,
            ControlFlow::Break(Result::Err(err)) => Some(err),
        }
    }
}

/// A trait for converting results into tracing results with custom log messages.
///
/// This trait extends [`Result<T, E>`] with methods that attach custom messages at various
/// log levels. When the resulting [`TracingResult`] is used with the `?` operator, the message
/// will be automatically logged via the [`tracing`] macro corresponding to the chosen level.
///
/// # Method Families
///
/// - **`or_*`** (`or_warn`, `or_error`, `or_debug`, `or_trace`): Attach a message that logs when **Err** is unpacked
/// - **`and_*`** (`and_warn`, `and_error`, `and_debug`, `and_trace`): Attach a message that logs when **Ok** is unpacked
///
/// # Example
///
/// ```
/// use std::io;
/// use tracing_result::Trace;
///
/// fn might_fail() -> io::Result<i32> {
///     Err(io::Error::other("network error"))
/// }
///
/// fn process() -> io::Result<i32> {
///     // Logs "Failed to fetch data" at WARN level if might_fail() returns Err
///     let result = might_fail().or_warn("Failed to fetch data");
///     result?;
///     Ok(42)
/// }
///
/// fn unexpected() -> io::Result<i32> {
///     // Logs "Unexpected success" at WARN level when Ok is unpacked
///     let res = io::Result::Ok(42).and_warn("Unexpected success")?;
///     Ok(res)
/// }
/// ```
pub trait Trace<T: Try> {
    /// Attaches a warning message to this result.
    ///
    /// Converts the [`Result`] into a [`TracingResult`] that will log the
    /// provided message via [`tracing::warn`] if an error occurs and is
    /// propagated with the `?` operator.
    ///
    /// # Example
    ///
    /// ```
    /// use std::io;
    /// use tracing_result::Trace;
    ///
    /// let result: io::Result<i32> = Err(io::Error::other("io error"));
    /// let tracing_result = result.or_warn("File read failed");
    ///
    /// // When tracing_result? is used, "File read failed" will be logged
    /// ```
    fn or_warn(self, name: &'static str) -> Traced<T>;

    /// Attaches a warning message to this result that logs when Ok is unpacked.
    ///
    /// Converts the [`Result`] into a [`TracingResult`] that will log the
    /// provided message via [`tracing::warn`] if the result is [`Ok`] and is
    /// propagated with the `?` operator.
    ///
    /// # Example
    ///
    /// ```
    /// use std::io;
    /// use tracing_result::Trace;
    ///
    /// let result: io::Result<i32> = Ok(42);
    /// let tracing_result = result.and_warn("Unexpected success");
    ///
    /// // When tracing_result? is used, "Unexpected success" will be logged
    /// ```
    fn and_warn(self, name: &'static str) -> Traced<T>;

    /// Attaches an error-level message to this result.
    ///
    /// Converts the [`Result`] into a [`TracingResult`] that will log the
    /// provided message via [`tracing::error`] if an error occurs and is
    /// propagated with the `?` operator.
    ///
    /// # Example
    ///
    /// ```
    /// use std::io;
    /// use tracing_result::Trace;
    ///
    /// let result: io::Result<i32> = Err(io::Error::other("io error"));
    /// let tracing_result = result.or_error("Critical failure");
    ///
    /// // When tracing_result? is used, "Critical failure" will be logged at ERROR level
    /// ```
    fn or_error(self, name: &'static str) -> Traced<T>;

    /// Attaches an error-level message to this result that logs when Ok is unpacked.
    ///
    /// Converts the [`Result`] into a [`TracingResult`] that will log the
    /// provided message via [`tracing::error`] if the result is [`Ok`] and is
    /// propagated with the `?` operator.
    ///
    /// # Example
    ///
    /// ```
    /// use std::io;
    /// use tracing_result::Trace;
    ///
    /// let result: io::Result<i32> = Ok(42);
    /// let tracing_result = result.and_error("This should not succeed");
    ///
    /// // When tracing_result? is used, "This should not succeed" will be logged at ERROR level
    /// ```
    fn and_error(self, name: &'static str) -> Traced<T>;

    /// Attaches a debug-level message to this result.
    ///
    /// Converts the [`Result`] into a [`TracingResult`] that will log the
    /// provided message via [`tracing::debug`] if an error occurs and is
    /// propagated with the `?` operator.
    ///
    /// # Example
    ///
    /// ```
    /// use std::io;
    /// use tracing_result::Trace;
    ///
    /// let result: io::Result<i32> = Err(io::Error::other("io error"));
    /// let tracing_result = result.or_debug("Debug: operation failed");
    ///
    /// // When tracing_result? is used, "Debug: operation failed" will be logged at DEBUG level
    /// ```
    fn or_debug(self, name: &'static str) -> Traced<T>;

    /// Attaches a debug-level message to this result that logs when Ok is unpacked.
    ///
    /// Converts the [`Result`] into a [`TracingResult`] that will log the
    /// provided message via [`tracing::debug`] if the result is [`Ok`] and is
    /// propagated with the `?` operator.
    ///
    /// # Example
    ///
    /// ```
    /// use std::io;
    /// use tracing_result::Trace;
    ///
    /// let result: io::Result<i32> = Ok(42);
    /// let tracing_result = result.and_debug("Debug: operation succeeded");
    ///
    /// // When tracing_result? is used, "Debug: operation succeeded" will be logged at DEBUG level
    /// ```
    fn and_debug(self, name: &'static str) -> Traced<T>;

    /// Attaches a trace-level message to this result.
    ///
    /// Converts the [`Result`] into a [`TracingResult`] that will log the
    /// provided message via [`tracing::trace`] if an error occurs and is
    /// propagated with the `?` operator.
    ///
    /// # Example
    ///
    /// ```
    /// use std::io;
    /// use tracing_result::Trace;
    ///
    /// let result: io::Result<i32> = Err(io::Error::other("io error"));
    /// let tracing_result = result.or_trace("Trace: minor issue detected");
    ///
    /// // When tracing_result? is used, "Trace: minor issue detected" will be logged at TRACE level
    /// ```
    fn or_trace(self, name: &'static str) -> Traced<T>;

    /// Attaches a trace-level message to this result that logs when Ok is unpacked.
    ///
    /// Converts the [`Result`] into a [`TracingResult`] that will log the
    /// provided message via [`tracing::trace`] if the result is [`Ok`] and is
    /// propagated with the `?` operator.
    ///
    /// # Example
    ///
    /// ```
    /// use std::io;
    /// use tracing_result::Trace;
    ///
    /// let result: io::Result<i32> = Ok(42);
    /// let tracing_result = result.and_trace("Trace: operation completed");
    ///
    /// // When tracing_result? is used, "Trace: operation completed" will be logged at TRACE level
    /// ```
    fn and_trace(self, name: &'static str) -> Traced<T>;
}

impl<T: Try> Trace<T> for T {
    fn or_warn(self, name: &'static str) -> Traced<T> {
        Traced {
            inner: self,
            event: TracingConfig::OnResidual(Event {
                level: Level::WARN,
                message: name,
            }),
        }
    }

    fn and_warn(self, name: &'static str) -> Traced<T> {
        Traced {
            inner: self,
            event: TracingConfig::OnOutput(Event {
                level: Level::WARN,
                message: name,
            }),
        }
    }

    fn or_error(self, name: &'static str) -> Traced<T> {
        Traced {
            inner: self,
            event: TracingConfig::OnResidual(Event {
                level: Level::ERROR,
                message: name,
            }),
        }
    }

    fn and_error(self, name: &'static str) -> Traced<T> {
        Traced {
            inner: self,
            event: TracingConfig::OnOutput(Event {
                level: Level::ERROR,
                message: name,
            }),
        }
    }

    fn or_debug(self, name: &'static str) -> Traced<T> {
        Traced {
            inner: self,
            event: TracingConfig::OnResidual(Event {
                level: Level::DEBUG,
                message: name,
            }),
        }
    }

    fn and_debug(self, name: &'static str) -> Traced<T> {
        Traced {
            inner: self,
            event: TracingConfig::OnOutput(Event {
                level: Level::DEBUG,
                message: name,
            }),
        }
    }

    fn or_trace(self, name: &'static str) -> Traced<T> {
        Traced {
            inner: self,
            event: TracingConfig::OnResidual(Event {
                level: Level::TRACE,
                message: name,
            }),
        }
    }

    fn and_trace(self, name: &'static str) -> Traced<T> {
        Traced {
            inner: self,
            event: TracingConfig::OnOutput(Event {
                level: Level::TRACE,
                message: name,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io;

    use tracing_test::traced_test;

    use super::*;

    /// Tests that `or_warn` on an Ok result does not log.
    #[traced_test]
    #[test]
    fn or_warn_ok() {
        fn no_error() -> io::Result<()> {
            io::Result::Ok(()).or_warn("stuff")?;
            Ok(())
        }

        assert!(no_error().is_ok());
        assert!(!logs_contain("stuff"));
    }

    /// Tests that `or_warn` on an Err result logs the warning message.
    #[traced_test]
    #[test]
    fn or_warn_err() {
        fn err() -> io::Result<()> {
            Err(io::Error::other("oops")).or_warn("stuff")?;
            Ok(())
        }

        assert!(err().is_err());
        assert!(!logs_contain("name"));
        assert!(logs_contain("stuff"));
    }

    /// Tests that `and_warn` on an Ok result logs the warning message.
    #[traced_test]
    #[test]
    fn and_warn_ok() {
        fn ok() -> io::Result<()> {
            io::Result::Ok(()).and_warn("ok warn")?;
            Ok(())
        }

        assert!(ok().is_ok());
        assert!(logs_contain("ok warn"));
    }

    /// Tests that `and_warn` on an Err result does not log.
    #[traced_test]
    #[test]
    fn and_warn_err() {
        fn err() -> io::Result<()> {
            Err(io::Error::other("oops")).and_warn("should not log")?;
            Ok(())
        }

        assert!(err().is_err());
        assert!(!logs_contain("should not log"));
    }

    /// Tests that `or_error` on an Ok result does not log.
    #[traced_test]
    #[test]
    fn or_error_ok() {
        fn no_error() -> io::Result<()> {
            io::Result::Ok(()).or_error("should not log")?;
            Ok(())
        }

        assert!(no_error().is_ok());
        assert!(!logs_contain("should not log"));
    }

    /// Tests that `or_error` on an Err result logs at ERROR level.
    #[traced_test]
    #[test]
    fn or_error_err() {
        fn err() -> io::Result<()> {
            Err(io::Error::other("oops")).or_error("error message")?;
            Ok(())
        }

        assert!(err().is_err());
        assert!(logs_contain("error message"));
    }

    /// Tests that `and_error` on an Ok result logs at ERROR level.
    #[traced_test]
    #[test]
    fn and_error_ok() {
        fn ok() -> io::Result<()> {
            io::Result::Ok(()).and_error("error on ok")?;
            Ok(())
        }

        assert!(ok().is_ok());
        assert!(logs_contain("error on ok"));
    }

    /// Tests that `and_error` on an Err result does not log.
    #[traced_test]
    #[test]
    fn and_error_err() {
        fn err() -> io::Result<()> {
            Err(io::Error::other("oops")).and_error("should not log")?;
            Ok(())
        }

        assert!(err().is_err());
        assert!(!logs_contain("should not log"));
    }

    /// Tests that `or_debug` on an Ok result does not log.
    #[traced_test]
    #[test]
    fn or_debug_ok() {
        fn no_error() -> io::Result<()> {
            io::Result::Ok(()).or_debug("should not log")?;
            Ok(())
        }

        assert!(no_error().is_ok());
        assert!(!logs_contain("should not log"));
    }

    /// Tests that `or_debug` on an Err result logs at DEBUG level.
    #[traced_test]
    #[test]
    fn or_debug_err() {
        fn err() -> io::Result<()> {
            Err(io::Error::other("oops")).or_debug("debug message")?;
            Ok(())
        }

        assert!(err().is_err());
        assert!(logs_contain("debug message"));
    }

    /// Tests that `and_debug` on an Ok result logs at DEBUG level.
    #[traced_test]
    #[test]
    fn and_debug_ok() {
        fn ok() -> io::Result<()> {
            io::Result::Ok(()).and_debug("debug on ok")?;
            Ok(())
        }

        assert!(ok().is_ok());
        assert!(logs_contain("debug on ok"));
    }

    /// Tests that `and_debug` on an Err result does not log.
    #[traced_test]
    #[test]
    fn and_debug_err() {
        fn err() -> io::Result<()> {
            Err(io::Error::other("oops")).and_debug("should not log")?;
            Ok(())
        }

        assert!(err().is_err());
        assert!(!logs_contain("should not log"));
    }

    /// Tests that `or_trace` on an Ok result does not log.
    #[traced_test]
    #[test]
    fn or_trace_ok() {
        fn no_error() -> io::Result<()> {
            io::Result::Ok(()).or_trace("should not log")?;
            Ok(())
        }

        assert!(no_error().is_ok());
        assert!(!logs_contain("should not log"));
    }

    /// Tests that `or_trace` on an Err result logs at TRACE level.
    #[traced_test]
    #[test]
    fn or_trace_err() {
        fn err() -> io::Result<()> {
            Err(io::Error::other("oops")).or_trace("trace message")?;
            Ok(())
        }

        assert!(err().is_err());
        assert!(logs_contain("trace message"));
    }

    /// Tests that `and_trace` on an Ok result logs at TRACE level.
    #[traced_test]
    #[test]
    fn and_trace_ok() {
        fn ok() -> io::Result<()> {
            io::Result::Ok(()).and_trace("trace on ok")?;
            Ok(())
        }

        assert!(ok().is_ok());
        assert!(logs_contain("trace on ok"));
    }

    /// Tests that `and_trace` on an Err result does not log.
    #[traced_test]
    #[test]
    fn and_trace_err() {
        fn err() -> io::Result<()> {
            Err(io::Error::other("oops")).and_trace("should not log")?;
            Ok(())
        }

        assert!(err().is_err());
        assert!(!logs_contain("should not log"));
    }

    /// Tests that `ok` on an Ok(v) with no log entry returns Some(v) and does not emit.
    #[traced_test]
    #[test]
    fn ok_or_ok() {
        let good = io::Result::Ok(5).or_warn("should not log");
        let v = good.ok();
        assert_eq!(v, Some(5));
        assert!(!logs_contain("should not log"));
    }

    #[traced_test]
    #[test]
    fn ok_and_ok() {
        let good = io::Result::Ok(5).and_warn("should log");
        let v = good.ok();
        assert_eq!(v, Some(5));
        assert!(logs_contain("should log"));
    }

    #[traced_test]
    #[test]
    fn err_or_err() {
        let err: TracingResult<i32, _> = Err(io::Error::other("oops")).or_warn("should log");
        let v = err.err();
        assert_eq!(v.map(|e| e.to_string()), Some("oops".to_string()));
        assert!(logs_contain("should log"));
    }

    #[traced_test]
    #[test]
    fn err_and_err() {
        let err: TracingResult<i32, _> = Err(io::Error::other("oops")).and_warn("should not log");
        let v = err.err();
        assert_eq!(v.map(|e| e.to_string()), Some("oops".to_string()));
        assert!(!logs_contain("should not log"));
    }

    #[traced_test]
    #[test]
    fn option() {
        fn opt() -> Option<usize> {
            let n = Some(5).or_warn("wibble")?;
            Some(n)
        }

        assert_eq!(opt(), Some(5));
        assert!(!logs_contain("wibble"));
    }

    #[traced_test]
    #[test]
    fn none_and_then() {
        let _: Option<usize> = None.or_warn("wibble").and_then(|x| Some(x + 1));
        assert!(logs_contain("wibble"));
    }

    #[traced_test]
    #[test]
    fn some_and_then() {
        let n = Some(4).or_warn("wibble").and_then(|x| Some(x + 1));
        assert_eq!(n, Some(5));
        assert!(!logs_contain("wibble"));
    }
}
