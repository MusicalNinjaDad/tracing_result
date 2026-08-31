use std::io;

use tracing_result::*;
use tracing_subscriber::{filter::LevelFilter, fmt::layer, prelude::*, registry};

fn main() -> io::Result<()> {
    let stdout = layer().pretty().with_filter(LevelFilter::TRACE);
    registry().with(stdout).init();

    Err(io::Error::other("Oopsie")).and_warn("boom!")?;

    Ok(())
}
