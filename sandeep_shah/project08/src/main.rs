use anyhow::{bail, Result};
use clap::Parser;
use csvprof::cli::{Cli, OutputFormat};
use csvprof::profiler::{profile_csv, ProfileConfig};
use csvprof::report::{Formatter, JsonFormatter, TextFormatter};

fn main() -> Result<()> {
    let args = Cli::parse();

    if !args.delimiter.is_ascii() {
        bail!("delimiter must be a single ASCII character");
    }

    let cfg = ProfileConfig {
        delimiter: args.delimiter as u8,
        has_headers: !args.no_headers,
        percentiles: args.percentiles,
        histogram: args.histogram,
        max_categories: args.max_categories,
        categorical_ratio: args.categorical_ratio,
    };

    let report = profile_csv(&args.file, &cfg)?;

    let formatter: Box<dyn Formatter> = match args.format {
        OutputFormat::Text => Box::new(TextFormatter),
        OutputFormat::Json => Box::new(JsonFormatter),
    };

    let output = formatter.format(&report)?;
    println!("{output}");

    Ok(())
}