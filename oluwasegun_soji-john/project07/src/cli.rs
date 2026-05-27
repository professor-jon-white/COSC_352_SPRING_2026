use std::collections::HashSet;

use clap::{Parser, ValueHint};

use crate::profile::ProfilerConfig;

#[derive(Debug, Parser)]
#[command(
    name = "csvprof",
    version,
    about = "Stream a CSV file and produce a column-by-column profiling report."
)]
pub struct Cli {
    #[arg(value_name = "FILE", value_hint = ValueHint::FilePath)]
    pub file: String,

    #[arg(
        short = 'd',
        long = "delimiter",
        default_value = ",",
        value_parser = parse_delimiter,
        help = "Single-character field delimiter. Use '\\t' for tabs."
    )]
    pub delimiter: u8,

    #[arg(long, help = "Treat the first row as data instead of column headers.")]
    pub no_headers: bool,

    #[arg(long, help = "Show p5, p25, p75, and p95 for numeric columns.")]
    pub percentiles: bool,

    #[arg(
        long,
        help = "Show an ASCII frequency histogram for categorical columns."
    )]
    pub histogram: bool,

    #[arg(
        long,
        default_value_t = 20,
        help = "Maximum unique-value count that still counts as low-cardinality categorical data."
    )]
    pub categorical_threshold: usize,

    #[arg(
        long,
        default_value_t = 5,
        help = "How many most/least frequent values to show for categorical and boolean columns."
    )]
    pub max_frequencies: usize,

    #[arg(
        long = "null",
        value_delimiter = ',',
        default_values_t = default_null_markers(),
        help = "Extra null markers, comma-separated. Blank cells are always treated as null."
    )]
    pub null_markers: Vec<String>,
}

impl Cli {
    pub fn to_config(&self) -> ProfilerConfig {
        let null_markers = self
            .null_markers
            .iter()
            .map(|marker| marker.trim().to_ascii_lowercase())
            .filter(|marker| !marker.is_empty())
            .collect::<HashSet<_>>();

        ProfilerConfig {
            delimiter: self.delimiter,
            has_headers: !self.no_headers,
            include_percentiles: self.percentiles,
            include_histogram: self.histogram,
            categorical_threshold: self.categorical_threshold,
            max_frequencies: self.max_frequencies,
            null_markers,
        }
    }
}

fn default_null_markers() -> Vec<String> {
    vec![
        "na".to_string(),
        "n/a".to_string(),
        "null".to_string(),
        "none".to_string(),
        "nil".to_string(),
    ]
}

fn parse_delimiter(input: &str) -> Result<u8, String> {
    match input {
        "\\t" => Ok(b'\t'),
        _ if input.chars().count() == 1 => Ok(input.as_bytes()[0]),
        _ => Err("delimiter must be exactly one character or the literal \\t".to_string()),
    }
}
