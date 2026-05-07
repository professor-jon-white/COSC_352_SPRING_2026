use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufReader, Read};
use std::path::PathBuf;

use anyhow::{Context, Result};
use chrono::{DateTime, NaiveDate};
use clap::Parser;
use csv::Reader;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Parser)]
#[command(name = "csvprof")]
#[command(about = "A command-line CSV data profiling tool")]
struct Args {
    /// Path to input CSV file (use '-' for stdin)
    file: PathBuf,

    /// Include percentiles in numeric statistics
    #[arg(long)]
    percentiles: bool,

    /// Include frequency histograms for categorical columns
    #[arg(long)]
    histogram: bool,

    /// Output format (table or json)
    #[arg(long, default_value = "table")]
    format: OutputFormat,
}

#[derive(Clone, Debug)]
enum OutputFormat {
    Table,
    Json,
}

impl std::str::FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "table" => Ok(OutputFormat::Table),
            "json" => Ok(OutputFormat::Json),
            _ => Err(format!("Invalid output format: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
enum DataType {
    Integer,
    Float,
    Boolean,
    Date,
    Categorical,
    Text,
}

impl std::fmt::Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataType::Integer => write!(f, "Integer"),
            DataType::Float => write!(f, "Float"),
            DataType::Boolean => write!(f, "Boolean"),
            DataType::Date => write!(f, "Date"),
            DataType::Categorical => write!(f, "Categorical"),
            DataType::Text => write!(f, "Text"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ColumnStats {
    column_name: String,
    data_type: DataType,
    row_count: usize,
    null_count: usize,
    null_percentage: f64,
    unique_count: usize,

    // Numeric/Date specific
    min_value: Option<String>,
    max_value: Option<String>,
    mean: Option<f64>,
    median: Option<f64>,
    std_dev: Option<f64>,
    percentiles: Option<Percentiles>,

    // Categorical specific
    top_values: Option<Vec<(String, usize)>>,
    histogram: Option<HashMap<String, usize>>,

    // Text specific
    min_length: Option<usize>,
    max_length: Option<usize>,

    // Quality warnings
    warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Percentiles {
    p5: f64,
    p25: f64,
    p75: f64,
    p95: f64,
}

#[derive(Error, Debug)]
enum ProfileError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("CSV parsing error: {0}")]
    Csv(#[from] csv::Error),

    #[error("Invalid data type inference")]
    TypeInference,

    #[error("Column '{0}' has mixed types")]
    MixedTypes(String),
}

fn main() -> Result<()> {
    let args = Args::parse();

    let reader = get_reader(&args.file)?;
    let stats = profile_csv(reader, args.percentiles, args.histogram)?;

    match args.format {
        OutputFormat::Table => print_table_report(&stats),
        OutputFormat::Json => print_json_report(&stats)?,
    }

    Ok(())
}

fn get_reader(file_path: &PathBuf) -> Result<Reader<Box<dyn Read>>> {
    let reader: Box<dyn Read> = if file_path == &PathBuf::from("-") {
        Box::new(io::stdin())
    } else {
        Box::new(BufReader::new(File::open(file_path).context("Failed to open file")?))
    };

    Ok(csv::Reader::from_reader(reader))
}

fn profile_csv<R: Read>(
    mut reader: Reader<R>,
    include_percentiles: bool,
    include_histogram: bool,
) -> Result<Vec<ColumnStats>> {
    let headers = reader.headers()?.clone();
    let column_count = headers.len();

    // Initialize collectors for each column
    let mut column_collectors: Vec<ColumnCollector> = (0..column_count)
        .map(|_| ColumnCollector::new())
        .collect();

    // Process each row
    for result in reader.records() {
        let record = result?;
        for (i, field) in record.iter().enumerate() {
            if i < column_collectors.len() {
                column_collectors[i].add_value(field);
            }
        }
    }

    // Generate statistics for each column
    let mut stats = Vec::new();
    for (i, collector) in column_collectors.into_iter().enumerate() {
        let column_name = headers.get(i).unwrap_or(&format!("Column {}", i)).to_string();
        let stat = collector.compute_stats(
            &column_name,
            include_percentiles,
            include_histogram,
        )?;
        stats.push(stat);
    }

    Ok(stats)
}

#[derive(Debug)]
struct ColumnCollector {
    values: Vec<String>,
    null_count: usize,
}

impl ColumnCollector {
    fn new() -> Self {
        Self {
            values: Vec::new(),
            null_count: 0,
        }
    }

    fn add_value(&mut self, value: &str) {
        if value.trim().is_empty() {
            self.null_count += 1;
        } else {
            self.values.push(value.to_string());
        }
    }

    fn compute_stats(
        self,
        column_name: &str,
        include_percentiles: bool,
        include_histogram: bool,
    ) -> Result<ColumnStats> {
        let row_count = self.values.len() + self.null_count;
        let null_percentage = if row_count > 0 {
            (self.null_count as f64 / row_count as f64) * 100.0
        } else {
            0.0
        };

        let unique_count = self.values.iter().collect::<std::collections::HashSet<_>>().len();

        // Infer data type
        let data_type = infer_data_type(&self.values)?;

        let mut warnings = Vec::new();

        // Check for constant column
        if unique_count == 1 && self.null_count == 0 {
            warnings.push("Constant column (all values identical)".to_string());
        }

        // Check for low cardinality categorical
        if matches!(data_type, DataType::Categorical) && unique_count <= 3 {
            warnings.push(format!("Low cardinality categorical ({} unique values)", unique_count));
        }

        match data_type {
            DataType::Integer | DataType::Float => {
                self.compute_numeric_stats(
                    column_name,
                    data_type,
                    row_count,
                    null_percentage,
                    unique_count,
                    include_percentiles,
                    warnings,
                )
            }
            DataType::Boolean => {
                self.compute_categorical_stats(
                    column_name,
                    data_type,
                    row_count,
                    null_percentage,
                    unique_count,
                    include_histogram,
                    warnings,
                )
            }
            DataType::Date => {
                self.compute_date_stats(
                    column_name,
                    row_count,
                    null_percentage,
                    unique_count,
                    warnings,
                )
            }
            DataType::Categorical | DataType::Text => {
                self.compute_categorical_stats(
                    column_name,
                    data_type,
                    row_count,
                    null_percentage,
                    unique_count,
                    include_histogram,
                    warnings,
                )
            }
        }
    }

    fn compute_numeric_stats(
        self,
        column_name: &str,
        data_type: DataType,
        row_count: usize,
        null_percentage: f64,
        unique_count: usize,
        include_percentiles: bool,
        warnings: Vec<String>,
    ) -> Result<ColumnStats> {
        let numeric_values: Vec<f64> = self.values
            .iter()
            .filter_map(|v| v.parse::<f64>().ok())
            .collect();

        if numeric_values.is_empty() {
            return Ok(ColumnStats {
                column_name: column_name.to_string(),
                data_type,
                row_count,
                null_count: self.null_count,
                null_percentage,
                unique_count,
                min_value: None,
                max_value: None,
                mean: None,
                median: None,
                std_dev: None,
                percentiles: None,
                top_values: None,
                histogram: None,
                min_length: None,
                max_length: None,
                warnings,
            });
        }

        let min_value = numeric_values.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).copied();
        let max_value = numeric_values.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).copied();

        let mean = Some(numeric_values.iter().sum::<f64>() / numeric_values.len() as f64);

        let mut sorted_values = numeric_values.clone();
        sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let median = Some(sorted_values[sorted_values.len() / 2]);

        let variance = numeric_values.iter()
            .map(|v| (v - mean.unwrap()).powi(2))
            .sum::<f64>() / numeric_values.len() as f64;
        let std_dev = Some(variance.sqrt());

        let percentiles = if include_percentiles {
            Some(compute_percentiles(&sorted_values))
        } else {
            None
        };

        Ok(ColumnStats {
            column_name: column_name.to_string(),
            data_type,
            row_count,
            null_count: self.null_count,
            null_percentage,
            unique_count,
            min_value: min_value.map(|v| v.to_string()),
            max_value: max_value.map(|v| v.to_string()),
            mean,
            median,
            std_dev,
            percentiles,
            top_values: None,
            histogram: None,
            min_length: None,
            max_length: None,
            warnings,
        })
    }

    fn compute_date_stats(
        self,
        column_name: &str,
        row_count: usize,
        null_percentage: f64,
        unique_count: usize,
        warnings: Vec<String>,
    ) -> Result<ColumnStats> {
        let date_values: Vec<NaiveDate> = self.values
            .iter()
            .filter_map(|v| NaiveDate::parse_from_str(v, "%Y-%m-%d").ok())
            .collect();

        let min_value = date_values.iter().min().map(|d| d.to_string());
        let max_value = date_values.iter().max().map(|d| d.to_string());

        Ok(ColumnStats {
            column_name: column_name.to_string(),
            data_type: DataType::Date,
            row_count,
            null_count: self.null_count,
            null_percentage,
            unique_count,
            min_value,
            max_value,
            mean: None,
            median: None,
            std_dev: None,
            percentiles: None,
            top_values: None,
            histogram: None,
            min_length: None,
            max_length: None,
            warnings,
        })
    }

    fn compute_categorical_stats(
        self,
        column_name: &str,
        data_type: DataType,
        row_count: usize,
        null_percentage: f64,
        unique_count: usize,
        include_histogram: bool,
        warnings: Vec<String>,
    ) -> Result<ColumnStats> {
        let mut freq_map: HashMap<String, usize> = HashMap::new();
        for value in &self.values {
            *freq_map.entry(value.clone()).or_insert(0) += 1;
        }

        let mut freq_vec: Vec<(String, usize)> = freq_map.into_iter().collect();
        freq_vec.sort_by(|a, b| b.1.cmp(&a.1));

        let top_values = if freq_vec.len() >= 5 {
            Some(freq_vec[..5].to_vec())
        } else {
            Some(freq_vec.clone())
        };

        let histogram = if include_histogram {
            let mut hist = HashMap::new();
            for (value, count) in &freq_vec {
                hist.insert(value.clone(), *count);
            }
            Some(hist)
        } else {
            None
        };

        let (min_length, max_length) = if matches!(data_type, DataType::Text) {
            let lengths: Vec<usize> = self.values.iter().map(|v| v.len()).collect();
            (Some(*lengths.iter().min().unwrap_or(&0)), Some(*lengths.iter().max().unwrap_or(&0)))
        } else {
            (None, None)
        };

        Ok(ColumnStats {
            column_name: column_name.to_string(),
            data_type,
            row_count,
            null_count: self.null_count,
            null_percentage,
            unique_count,
            min_value: None,
            max_value: None,
            mean: None,
            median: None,
            std_dev: None,
            percentiles: None,
            top_values,
            histogram,
            min_length,
            max_length,
            warnings,
        })
    }
}

fn infer_data_type(values: &[String]) -> Result<DataType> {
    if values.is_empty() {
        return Ok(DataType::Text);
    }

    let mut type_counts = HashMap::new();

    for value in values {
        let inferred = infer_single_value(value);
        *type_counts.entry(inferred).or_insert(0) += 1;
    }

    // Find the most common type
    let (most_common_type, _) = type_counts
        .into_iter()
        .max_by_key(|&(_, count)| count)
        .unwrap_or((DataType::Text, 0));

    // If most common type is Text, check if it should be Categorical
    // Categorical if unique ratio is low (less than 50% unique)
    if matches!(most_common_type, DataType::Text) {
        let unique_count = values.iter().collect::<std::collections::HashSet<_>>().len();
        let unique_ratio = unique_count as f64 / values.len() as f64;
        if unique_ratio <= 0.5 && unique_count <= 20 {
            return Ok(DataType::Categorical);
        }
    }

    Ok(most_common_type)
}

fn infer_single_value(value: &str) -> DataType {
    // Try boolean first
    if matches!(value.to_lowercase().as_str(), "true" | "false" | "1" | "0" | "yes" | "no") {
        return DataType::Boolean;
    }

    // Try integer
    if value.parse::<i64>().is_ok() {
        return DataType::Integer;
    }

    // Try float
    if value.parse::<f64>().is_ok() {
        return DataType::Float;
    }

    // Try date
    if NaiveDate::parse_from_str(value, "%Y-%m-%d").is_ok()
        || DateTime::parse_from_rfc3339(value).is_ok() {
        return DataType::Date;
    }

    // Check if categorical (low unique ratio - but we'll handle this at column level)
    // For now, assume text
    DataType::Text
}

fn compute_percentiles(sorted_values: &[f64]) -> Percentiles {
    let len = sorted_values.len();
    Percentiles {
        p5: sorted_values[(len as f64 * 0.05) as usize],
        p25: sorted_values[(len as f64 * 0.25) as usize],
        p75: sorted_values[(len as f64 * 0.75) as usize],
        p95: sorted_values[(len as f64 * 0.95) as usize],
    }
}

fn print_table_report(stats: &[ColumnStats]) {
    for stat in stats {
        println!("Column: {}", stat.column_name);
        println!("Type: {}", stat.data_type);
        println!("Rows: {} (nulls: {} - {:.1}%)", stat.row_count, stat.null_count, stat.null_percentage);
        println!("Unique values: {}", stat.unique_count);

        if let (Some(min), Some(max)) = (&stat.min_value, &stat.max_value) {
            println!("Range: {} to {}", min, max);
        }

        if let Some(mean) = stat.mean {
            println!("Mean: {:.2}", mean);
        }

        if let Some(median) = stat.median {
            println!("Median: {:.2}", median);
        }

        if let Some(std_dev) = stat.std_dev {
            println!("Std Dev: {:.2}", std_dev);
        }

        if let Some(percentiles) = &stat.percentiles {
            println!("Percentiles: p5={:.2}, p25={:.2}, p75={:.2}, p95={:.2}",
                    percentiles.p5, percentiles.p25, percentiles.p75, percentiles.p95);
        }

        if let Some(top_values) = &stat.top_values {
            println!("Top values:");
            for (value, count) in top_values {
                println!("  {}: {}", value, count);
            }
        }

        if let (Some(min_len), Some(max_len)) = (stat.min_length, stat.max_length) {
            println!("String length: {} to {}", min_len, max_len);
        }

        if !stat.warnings.is_empty() {
            println!("Warnings:");
            for warning in &stat.warnings {
                println!("  - {}", warning);
            }
        }

        println!();
    }
}

fn print_json_report(stats: &[ColumnStats]) -> Result<()> {
    let json = serde_json::to_string_pretty(stats)?;
    println!("{}", json);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    #[test]
    fn test_basic_csv_profiling() {
        let output = Command::new("cargo")
            .args(&["run", "--", "../sample.csv"])
            .current_dir("/workspaces/COSC_352_SPRING_2026/csvprof")
            .output()
            .expect("Failed to execute command");

        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.contains("Column: name"));
        assert!(stdout.contains("Type: Text"));
        assert!(stdout.contains("Column: age"));
        assert!(stdout.contains("Type: Integer"));
        assert!(stdout.contains("Mean:"));
    }

    #[test]
    fn test_json_output() {
        let output = Command::new("cargo")
            .args(&["run", "--", "--format", "json", "../sample.csv"])
            .current_dir("/workspaces/COSC_352_SPRING_2026/csvprof")
            .output()
            .expect("Failed to execute command");

        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.contains(r#""data_type": "Text""#));
        assert!(stdout.contains(r#""data_type": "Integer""#));
    }
}
