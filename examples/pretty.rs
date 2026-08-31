use std::io;

use tracing::warn_span;
use tracing_result::*;
use tracing_subscriber::{filter::LevelFilter, fmt::layer, prelude::*, registry};

fn main() -> io::Result<()> {
    let stdout = layer().pretty().with_filter(LevelFilter::TRACE);
    registry().with(stdout).init();

    let answer = 42;

    warn_span!("pretty example", answer);
    Err(io::Error::other("Oopsie")).or_warn("boom!")?;

    Ok(())
}
