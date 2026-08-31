use tracing_result::*;
use tracing_subscriber::{filter::LevelFilter, fmt::layer, prelude::*, registry};

fn main() -> Result<(), String> {
    let stdout = layer().pretty().with_filter(LevelFilter::TRACE);
    registry().with(stdout).init();

    Err("Oopsie".to_string()).and_warn("boom!")?;

    Ok(())
}
