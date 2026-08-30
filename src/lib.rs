#![cfg_attr(unstable_never_type, feature(never_type))]
#![cfg_attr(unstable_try_trait_v2, feature(try_trait_v2))]
#![cfg_attr(unstable_try_trait_v2_residual, feature(try_trait_v2_residual))]

use std::ops::{ControlFlow, FromResidual, Residual, Try};

pub enum TracingResult<T, E> {
    Ok(T),
    Err { err: E, msg: String },
}

impl<T, E> Try for TracingResult<T, E> {
    type Output = T;

    type Residual = TracingResult<!, E>;

    fn from_output(output: Self::Output) -> Self {
        Self::Ok(output)
    }

    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self {
            TracingResult::Ok(val) => ControlFlow::Continue(val),
            TracingResult::Err { err, msg } => ControlFlow::Break(TracingResult::Err { err, msg }),
        }
    }
}

impl<T, E> FromResidual for TracingResult<T, E> {
    fn from_residual(residual: <Self as Try>::Residual) -> Self {
        todo!("from residual")
    }
}

impl<T, E> FromResidual<TracingResult<!, E>> for Result<T, E> {
    fn from_residual(residual: TracingResult<!, E>) -> Self {
        todo!("from residual for result")
    }
}

impl<T, E> Residual<T> for TracingResult<!, E> {
    type TryType = TracingResult<T, E>;
}

pub trait Trace<T, E> {
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
    use super::*;

    #[test]
    fn and_warn_ok() {
        fn no_error() -> Result<(), ()> {
            Ok(()).and_warn("stuff")?;
            Ok(())
        }

        assert!(no_error().is_ok());
    }
}
