# Project 07: `csvprof` CSV Profiling CLI (Rust)

## Overview

`csvprof` is a command-line data profiling tool written in Rust. It reads a CSV file row by row, infers the most likely type for each column, and prints a structured report about column shape, quality, and descriptive statistics.

The program is built to show:

- idiomatic Rust module design
- ownership-aware streaming CSV processing
- trait-based metric collection
- explicit error handling with `Result`
- extensibility through small reusable components

The tool does **not** load the entire CSV file into memory at once. Instead, it streams rows through a set of per-column profilers. It still keeps some column-level state in memory, such as frequency tables and numeric vectors for exact medians/percentiles, because those are needed to produce the report.

## Features

- Accepts a file path or `-` for stdin
- Supports configurable delimiter and header handling
- Infers these column types:
  - integer
  - float
  - boolean
  - date
  - categorical
  - text
- Computes per-column statistics:
  - row count
  - null count and null percentage
  - unique value count
  - min and max for numeric/date columns
  - mean, median, and standard deviation for numeric columns
  - optional p5, p25, p75, p95 percentiles
  - top / least frequent values for categorical and boolean columns
  - optional categorical histogram
  - shortest and longest string length for text-like columns
- Detects common quality issues:
  - mixed types
  - constant columns
  - low-cardinality categoricals
  - numeric outliers using the IQR rule

## CLI

```text
csvprof [OPTIONS] <FILE>
```

### Arguments

- `<FILE>`: path to a CSV file, or `-` to read from standard input

### Options

- `-d, --delimiter <CHAR>`: field delimiter, default `,`
- `--no-headers`: treat the first row as data
- `--percentiles`: show p5/p25/p75/p95 for numeric columns
- `--histogram`: show an ASCII histogram for categorical columns
- `--categorical-threshold <N>`: unique-value cutoff for low-cardinality categorical detection
- `--max-frequencies <N>`: number of most/least frequent values to show, default `5`
- `--null <LIST>`: additional comma-separated null markers, default `na,n/a,null,none,nil`

## Project Layout

- `src/main.rs` - program entry point
- `src/cli.rs` - command-line parsing with `clap`
- `src/error.rs` - custom error type
- `src/value.rs` - value parsing and primitive classification
- `src/stats.rs` - reusable metric accumulators
- `src/profile.rs` - streaming dataset and column profilers
- `src/report.rs` - human-readable report rendering
- `sample_data/employee_quality.csv` - demonstration dataset
- `run.sh` - convenience script for a sample run
- `Dockerfile` - containerized build/runtime

## How the Design Works

### 1. Streaming ingestion

The program uses the `csv` crate with a flexible reader and processes one `StringRecord` at a time. Each row is immediately sent into a vector of `ColumnProfiler` instances.

### 2. Trait-based metric collection

The `Metric` trait in `src/stats.rs` is implemented by:

- `TypeCounter`
- `NumericStats`
- `DateStats`
- `TextStats`
- `FrequencyTable`

Each non-null cell is inspected once, then every metric decides whether that value is relevant.

### 3. Type inference

Each column tracks how many non-null values looked like:

- integers
- floats
- booleans
- dates
- general text

At the end of the scan, the column uses simple heuristics to choose its final type. If the values are mostly numeric but include a few invalid strings, the report still marks the column as numeric and emits a mixed-type warning.

### 4. Report generation

After profiling is complete, `report.rs` renders a readable terminal report with a dataset summary followed by a detailed section for each column.

## Build and Run

From this folder:

```bash
cargo run -- sample_data/employee_quality.csv --percentiles --histogram
```

Or use the helper script:

```bash
./run.sh
```

To profile stdin:

```bash
cat sample_data/employee_quality.csv | cargo run -- - --percentiles
```

## Example Output

```text
CSV Profile Report
==================

Dataset Summary
---------------
Source: sample_data/employee_quality.csv
Rows: 12
Columns: 9
Delimiter: ,
Percentiles: enabled
Categorical histogram: enabled
```

Each column section then shows type inference, null counts, unique counts, warnings, and any statistics relevant to that inferred type.

## Docker

Build the image:

```bash
docker build -t csvprof-project07 .
```

Run the sample profile:

```bash
docker run --rm -v "$PWD/sample_data:/data" csvprof-project07 /data/employee_quality.csv --percentiles --histogram
```

## Assumptions and Tradeoffs

- Blank cells are always null.
- Extra null markers are matched case-insensitively.
- Dates are inferred from several common formats, including ISO-8601 and U.S. slash formats.
- Exact medians and percentiles are computed by storing numeric values per numeric column.
- Unique counts and categorical frequencies are exact because the tool keeps per-column frequency tables.
- For malformed rows with missing fields, missing cells are treated as nulls.
- If a later row has more fields than the header row, the tool automatically creates `column_N` names for the extra columns.

## Why This Fits The Assignment

This program is a fully working CLI tool with:

- valid report output
- streaming CSV processing
- extensible Rust code split into modules
- traits and ownership-based design
- explicit error handling
- statistics and quality checks required by the project prompt
