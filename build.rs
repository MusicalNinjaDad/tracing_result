use build_safely::prelude::*;

fn main() -> Result<()> {
    let ac = AutoCfg::new()?;

    // check to see any  downstream crate has defined
    // `unstable.allow-features` in `.cargo/config.toml`.
    let allowed_features = cargo_allowed_features()?;

    Ok(())
}
