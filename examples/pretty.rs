use std::io;

use tracing::error_span;
use tracing_result::*;
use tracing_subscriber::{filter::LevelFilter, fmt::layer, prelude::*, registry};

fn main() -> io::Result<()> {
    let stdout = layer().pretty().with_filter(LevelFilter::TRACE);
    registry().with(stdout).init();

    let _span = error_span!("pretty examples").entered();
    Ok(42).and_warn("This will log a WARN on success")?;

    Ok(42).and_error("This will log an ERROR on success")?;

    Ok(42).and_debug("This will log a DEBUG on success")?;

    Ok(42).and_trace("This will log a TRACE on success")?;

    Err(io::Error::other("Oopsie")).or_warn("This will log a WARN on error")?;

    Err(io::Error::other("Critical")).or_error("This will log an ERROR on error")?;

    Err(io::Error::other("Debug issue")).or_debug("This will log a DEBUG on error")?;

    Err(io::Error::other("Trace issue")).or_trace("This will log a TRACE on error")?;

    Ok(())
}
