use std::io;

use tracing::{debug_span, error_span, info_span, trace_span, warn_span};
use tracing_result::*;
use tracing_subscriber::{filter::LevelFilter, fmt::layer, prelude::*, registry};

fn main() -> io::Result<()> {
    let stdout = layer().pretty().with_filter(LevelFilter::TRACE);
    registry().with(stdout).init();

    let answer = 42;

    // Demonstrate or_* methods - log on Err
    warn_span!("or_warn example", answer);
    Err(io::Error::other("Oopsie")).or_warn("This will log a WARN on error")?;

    // Demonstrate and_* methods - log on Ok
    info_span!("and_warn example");
    Ok(42).and_warn("This will log a WARN on success")?;

    // Demonstrate all log levels with or_* (error case)
    error_span!("or_error example");
    Err(io::Error::other("Critical")).or_error("This will log an ERROR on error")?;

    debug_span!("or_debug example");
    Err(io::Error::other("Debug issue")).or_debug("This will log a DEBUG on error")?;

    trace_span!("or_trace example");
    Err(io::Error::other("Trace issue")).or_trace("This will log a TRACE on error")?;

    // Demonstrate all log levels with and_* (success case)
    error_span!("and_error example");
    Ok(42).and_error("This will log an ERROR on success")?;

    debug_span!("and_debug example");
    Ok(42).and_debug("This will log a DEBUG on success")?;

    trace_span!("and_trace example");
    Ok(42).and_trace("This will log a TRACE on success")?;

    Ok(())
}
