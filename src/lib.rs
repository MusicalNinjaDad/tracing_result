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
//! use std::io;
//! use tracing_result::{Trace, TracingResult};
//!
//! fn might_fail() -> io::Result<u32> {
//!     Err(io::Error::other("something went wrong"))
//! }
//!
//! fn compute() -> io::Result<u32> {
//!     might_fail().or_warn("Failed to compute value")?;
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

use std::{
    error::Error,
    ops::{ControlFlow, FromResidual, Residual, Try},
};
use tracing::Level;

/// Configuration for tracing log level and message.
#[derive(Debug, Clone, Copy)]
pub struct TracingConfig {
    /// The tracing level to use when logging.
    pub level: Level,
    /// The message to log.
    pub message: &'static str,
}

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
pub enum TracingResult<T, E: Error> {
    /// Success case containing the result value and optional tracing configuration.
    Ok {
        val: T,
        config: Option<TracingConfig>,
    },
    /// Error case containing both the error and optional tracing configuration.
    ///
    /// If `config` is present, the message is logged at the specified level
    /// when the error is propagated using the `?` operator.
    Err {
        err: E,
        config: Option<TracingConfig>,
    },
}

impl<T, E: Error> Try for TracingResult<T, E> {
    type Output = T;

    type Residual = TracingResult<!, E>;

    fn from_output(output: Self::Output) -> Self {
        Self::Ok {
            val: output,
            config: None,
        }
    }

    /// Executes the `Try` branch operation, logging messages based on config.
    ///
    /// When the result is [`Ok`] with a config, the message is logged at the specified level.
    /// When the result is [`Err`] with a config, the message is logged at the specified level
    /// with error info.
    ///
    /// This is the mechanism that enables automatic tracing when using the `?` operator.
    #[track_caller]
    #[inline(always)]
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self {
            TracingResult::Ok { val, config } => {
                if let Some(cfg) = config {
                    match cfg.level {
                        Level::ERROR => tracing::error!("{}", cfg.message),
                        Level::WARN => tracing::warn!("{}", cfg.message),
                        Level::INFO => tracing::info!("{}", cfg.message),
                        Level::DEBUG => tracing::debug!("{}", cfg.message),
                        Level::TRACE => tracing::trace!("{}", cfg.message),
                    }
                }
                ControlFlow::Continue(val)
            }
            TracingResult::Err { err, config } => {
                if let Some(cfg) = config {
                    match cfg.level {
                        Level::ERROR => tracing::error!(error = err.to_string(), "{}", cfg.message),
                        Level::WARN => tracing::warn!(error = err.to_string(), "{}", cfg.message),
                        Level::INFO => tracing::info!(error = err.to_string(), "{}", cfg.message),
                        Level::DEBUG => tracing::debug!(error = err.to_string(), "{}", cfg.message),
                        Level::TRACE => tracing::trace!(error = err.to_string(), "{}", cfg.message),
                    }
                }
                ControlFlow::Break(TracingResult::Err { err, config })
            }
        }
    }
}

impl<T, E: Error> FromResidual for TracingResult<T, E> {
    fn from_residual(residual: <Self as Try>::Residual) -> Self {
        match residual {
            TracingResult::Ok { .. } => unreachable!(),
            TracingResult::Err { err, config } => Self::Err { err, config },
        }
    }
}

impl<T, E: Error> FromResidual<TracingResult<!, E>> for Result<T, E> {
    fn from_residual(residual: TracingResult<!, E>) -> Self {
        match residual {
            TracingResult::Ok { .. } => unreachable!(),
            TracingResult::Err { err, .. } => Result::Err(err),
        }
    }
}

impl<T, E: Error> Residual<T> for TracingResult<!, E> {
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
/// use std::io;
/// use tracing_result::Trace;
///
/// fn might_fail() -> io::Result<i32> {
///     Err(io::Error::other("network error"))
/// }
///
/// fn process() -> io::Result<i32> {
///     let result = might_fail().or_warn("Failed to fetch data");
///     // If might_fail() returns Err, "Failed to fetch data" will be logged
///     // when result is used with ?
///     result?;
///     Ok(42)
/// }
/// ```
pub trait Trace<T, E: Error> {
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
    fn or_warn(self, name: &'static str) -> TracingResult<T, E>;

    /// emits a WARN when OK unpacked via `?`
    fn and_warn(self, name: &'static str) -> TracingResult<T, E>;

    fn or_error(self, name: &'static str) -> TracingResult<T, E>;
    fn and_error(self, name: &'static str) -> TracingResult<T, E>;

    fn or_debug(self, name: &'static str) -> TracingResult<T, E>;
    fn and_debug(self, name: &'static str) -> TracingResult<T, E>;

    fn or_trace(self, name: &'static str) -> TracingResult<T, E>;
    fn and_trace(self, name: &'static str) -> TracingResult<T, E>;
}

impl<T, E: Error> Trace<T, E> for Result<T, E> {
    fn or_warn(self, name: &'static str) -> TracingResult<T, E> {
        match self {
            Ok(val) => TracingResult::Ok { val, config: None },
            Err(err) => TracingResult::Err {
                err,
                config: Some(TracingConfig {
                    level: Level::WARN,
                    message: name,
                }),
            },
        }
    }

    fn and_warn(self, name: &'static str) -> TracingResult<T, E> {
        match self {
            Ok(val) => TracingResult::Ok {
                val,
                config: Some(TracingConfig {
                    level: Level::WARN,
                    message: name,
                }),
            },
            Err(err) => TracingResult::Err { err, config: None },
        }
    }

    fn or_error(self, name: &'static str) -> TracingResult<T, E> {
        match self {
            Ok(val) => TracingResult::Ok { val, config: None },
            Err(err) => TracingResult::Err {
                err,
                config: Some(TracingConfig {
                    level: Level::ERROR,
                    message: name,
                }),
            },
        }
    }

    fn and_error(self, name: &'static str) -> TracingResult<T, E> {
        match self {
            Ok(val) => TracingResult::Ok {
                val,
                config: Some(TracingConfig {
                    level: Level::ERROR,
                    message: name,
                }),
            },
            Err(err) => TracingResult::Err { err, config: None },
        }
    }

    fn or_debug(self, name: &'static str) -> TracingResult<T, E> {
        match self {
            Ok(val) => TracingResult::Ok { val, config: None },
            Err(err) => TracingResult::Err {
                err,
                config: Some(TracingConfig {
                    level: Level::DEBUG,
                    message: name,
                }),
            },
        }
    }

    fn and_debug(self, name: &'static str) -> TracingResult<T, E> {
        match self {
            Ok(val) => TracingResult::Ok {
                val,
                config: Some(TracingConfig {
                    level: Level::DEBUG,
                    message: name,
                }),
            },
            Err(err) => TracingResult::Err { err, config: None },
        }
    }

    fn or_trace(self, name: &'static str) -> TracingResult<T, E> {
        match self {
            Ok(val) => TracingResult::Ok { val, config: None },
            Err(err) => TracingResult::Err {
                err,
                config: Some(TracingConfig {
                    level: Level::TRACE,
                    message: name,
                }),
            },
        }
    }

    fn and_trace(self, name: &'static str) -> TracingResult<T, E> {
        match self {
            Ok(val) => TracingResult::Ok {
                val,
                config: Some(TracingConfig {
                    level: Level::TRACE,
                    message: name,
                }),
            },
            Err(err) => TracingResult::Err { err, config: None },
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
            Ok(()).or_warn("stuff")?;
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
            Ok(()).and_warn("ok warn")?;
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
            Ok(()).or_error("should not log")?;
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
            Ok(()).and_error("error on ok")?;
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
            Ok(()).or_debug("should not log")?;
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
            Ok(()).and_debug("debug on ok")?;
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
            Ok(()).or_trace("should not log")?;
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
            Ok(()).and_trace("trace on ok")?;
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
}
