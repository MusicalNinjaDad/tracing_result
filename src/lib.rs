#![cfg_attr(unstable_never_type, feature(never_type))]
#![cfg_attr(unstable_try_trait_v2, feature(try_trait_v2))]
#![cfg_attr(unstable_try_trait_v2_residual, feature(try_trait_v2_residual))]

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn and_warn_ok {

        fn no_error() -> Result<(), ()> {
            Ok(()).and_warn("stuff")?;
            Ok(())
        }

        assert!(no_error().is_ok());

    }
}