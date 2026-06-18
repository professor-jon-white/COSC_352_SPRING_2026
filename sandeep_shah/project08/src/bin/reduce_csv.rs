use anyhow::{Context, Result};
use std::env;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 4 {
        eprintln!("Usage:");
        eprintln!("cargo run --bin reduce_csv -- <input.csv> <output.csv> <max_rows>");
        std::process::exit(1);
    }

    let input = &args[1];
    let output = &args[2];
    let max_rows: usize = args[3].parse().context("max_rows must be a number")?;

    let mut rdr = csv::ReaderBuilder::new()
        .flexible(true)
        .from_path(input)
        .with_context(|| format!("failed to open input file: {}", input))?;

    let mut wtr = csv::WriterBuilder::new()
        .from_path(output)
        .with_context(|| format!("failed to create output file: {}", output))?;

    let headers = rdr.headers()?.clone();
    wtr.write_record(&headers)?;

    let mut count = 0usize;

    for result in rdr.records() {
        if count >= max_rows {
            break;
        }

        match result {
            Ok(record) => {
                wtr.write_record(&record)?;
                count += 1;
            }
            Err(e) => {
                eprintln!("Skipping bad row: {}", e);
            }
        }
    }

    wtr.flush()?;
    println!("Wrote {} rows to {}", count, output);

    Ok(())
}
