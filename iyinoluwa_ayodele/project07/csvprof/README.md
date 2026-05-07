# CSV Profile Tool

A command-line data profiling tool written in Rust that ingests CSV files and produces structured reports describing the shape, quality, and statistical characteristics of each column.

## Features

- **Automatic Type Inference**: Detects Integer, Float, Boolean, Date, Categorical, and Text data types
- **Comprehensive Statistics**: Computes appropriate statistics for each data type
- **Data Quality Checks**: Identifies nulls, outliers, mixed types, and data quality issues
- **Memory Efficient**: Streams data without loading entire files into memory
- **Multiple Output Formats**: Human-readable tables and structured JSON
- **Flexible Input**: Reads from files or standard input

## Installation

```bash
cargo build --release
```

## Usage

```bash
csvprof [OPTIONS] <FILE>
```

### Arguments

- `<FILE>`: Path to input CSV file (use `-` for stdin)

### Options

- `--percentiles`: Include percentiles (p5, p25, p75, p95) in numeric statistics
- `--histogram`: Include frequency histograms for categorical columns
- `--format <FORMAT>`: Output format (`table` or `json`, default: `table`)

### Examples

```bash
# Basic profiling
csvprof data.csv

# Include percentiles and histograms
csvprof --percentiles --histogram data.csv

# JSON output
csvprof --format json data.csv

# Read from stdin
cat data.csv | csvprof -
```

## Column Statistics

### All Column Types
- **Inferred Type**: Automatically detected data type
- **Row Count**: Total number of rows
- **Null Count**: Number of null/empty values
- **Null Percentage**: Percentage of null values
- **Unique Count**: Number of unique values

### Numeric Types (Integer, Float)
- **Min/Max**: Minimum and maximum values
- **Mean**: Average value
- **Median**: Middle value when sorted
- **Standard Deviation**: Measure of value dispersion
- **Percentiles** (optional): 5th, 25th, 75th, and 95th percentiles

### Date Types
- **Min/Max**: Earliest and latest dates

### Categorical and Boolean Types
- **Top Values**: Most frequent values (up to 5)
- **Frequency Histogram** (optional): Complete value frequency distribution

### Text Types
- **String Length**: Minimum and maximum string lengths

## Data Quality Warnings

- **Constant Column**: All values are identical
- **Low Cardinality Categorical**: Very few unique values (≤3)
- **Mixed Types**: Column contains inconsistent data types

## Sample Output

```
Column: age
Type: Integer
Rows: 1000 (nulls: 23 - 2.3%)
Unique values: 45
Range: 18 to 75
Mean: 42.15
Median: 41.00
Std Dev: 12.34
Percentiles: p5=22.00, p25=32.00, p75=52.00, p95=68.00

Column: department
Type: Categorical
Rows: 1000 (nulls: 0 - 0.0%)
Unique values: 5
Top values:
  Engineering: 350
  Sales: 250
  Marketing: 200
  HR: 150
  Finance: 50

Column: description
Type: Text
Rows: 1000 (nulls: 5 - 0.5%)
Unique values: 987
String length: 10 to 500
```

## Dependencies

- `csv`: CSV parsing and reading
- `clap`: Command-line argument parsing
- `serde`: Serialization for JSON output
- `chrono`: Date parsing and handling
- `anyhow`: Error handling
- `thiserror`: Structured error types
- `regex`: Pattern matching for type inference

## Architecture

The tool demonstrates idiomatic Rust design patterns:

- **Traits and Generics**: Type-safe data type inference and statistics computation
- **Ownership and Borrowing**: Efficient memory management without cloning
- **Error Handling**: Comprehensive error types with `thiserror`
- **Zero-Cost Abstractions**: Iterator-based streaming without heap allocation overhead
- **Streaming Processing**: Processes CSV rows one at a time to handle large files

## Performance

- **Memory Usage**: O(1) memory complexity - only stores column statistics, not raw data
- **Time Complexity**: O(n) where n is the number of rows
- **Type Inference**: Single-pass inference with fallback to most common type
- **Statistics**: Efficient computation using online algorithms where possible