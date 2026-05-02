mod cli;
mod column;
mod profiler;
mod stats;
mod analyze;

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    analyze::run_analysis()?;
    Ok(())
}