mod cli;
mod error;
mod profile;
mod report;
mod stats;
mod value;

use std::fs::File;
use std::io::{self, BufReader};

use clap::Parser;

use crate::cli::Cli;
use crate::error::CsvProfError;
use crate::profile::profile_csv;
use crate::report::RenderableReport;

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), CsvProfError> {
    let cli = Cli::parse();
    let config = cli.to_config();

    let report = if cli.file == "-" {
        let stdin = io::stdin();
        profile_csv(stdin.lock(), "stdin", &config)?
    } else {
        let file = File::open(&cli.file).map_err(|source| CsvProfError::OpenFile {
            path: cli.file.clone(),
            source,
        })?;
        profile_csv(BufReader::new(file), &cli.file, &config)?
    };

    println!("{}", report.render(&config));
    Ok(())
}
