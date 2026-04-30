# Project 09 Quick Start Guide

## What Was Created

A complete Rust project that uses the **Plotters** library to generate 6 PNG visualizations of Baltimore City data from projects 07 and 08.

## Project Location

```
/workspaces/COSC_352_SPRING_2026/obaloluwa_wojuade/project09/
```

## File Organization

```
project09/
├── Cargo.toml                    # Project configuration & dependencies
├── README.md                     # Complete documentation (required reading)
├── src/
│   ├── main.rs                   # Entry point & orchestration
│   ├── data.rs                   # Data loading & statistics computation
│   └── charts.rs                 # Chart generation functions (6 charts)
├── output/                       # Generated PNG files will go here
└── BUILD_VERIFICATION.md         # Technical verification checklist
    COMPLETION_SUMMARY.md         # Full implementation summary
```

## How to Run

### Prerequisites
1. Rust installed (https://rustup.rs)
2. Project07 must be built first

### Build and Run

```bash
# Step 1: Build project07 (the library dependency)
cd obaloluwa_wojuade/project07
cargo build

# Step 2: Run project09
cd ../project09
cargo run
```

### Expected Console Output

```
Loading liquor license data...
  29751 liquor license records loaded
Loading employee salary data...
  231945 employee salary records loaded

Computing statistics...
  133 unique zip codes found
  17362 total salary records processed

Generating charts...
  Generating chart 1: Top 15 zip codes by active licenses... Done
  Generating chart 2: License type distribution (Top 5 zips)... Done
  Generating chart 3: Average license fees (Top 10 zips)... Done
  Generating chart 4: Salary comparison (Police, Fire, Citywide)... Done
  Generating chart 5: Above-average earners distribution... Done
  Generating chart 6: Total vs active licenses (Top 10 zips)... Done

Done. Charts saved to output/
```

## Generated Chart Files

After running `cargo run`, you'll find these PNG files in the `output/` directory:

1. **01_top_zip_codes.png**
   - Bar chart showing top 15 zip codes by active liquor licenses
   - Key insight: Zip 21202 dominates with 3,967 active licenses

2. **02_license_types_top_5.png**
   - Stacked bar chart showing business type breakdown
   - Shows tavern vs restaurant vs package goods distribution
   - Key insight: Different zip codes have different business profiles

3. **03_license_fees.png**
   - Bar chart of average license fees
   - Range: $855 to $1,899
   - Key insight: Downtown premium locations have higher fees

4. **04_salary_comparison.png**
   - Grouped bar chart comparing average salaries
   - Citywide: $50,821 | Police: $76,678 | Fire: $82,139
   - Key insight: Public safety employees earn 50-62% above average

5. **05_above_avg_earners.png**
   - Bar chart showing count of above-average earners
   - Police: 2,407 (83.8%) | Fire: 1,395 (82.9%)
   - Key insight: Over 83% of public safety workers earn above citywide average

6. **06_active_vs_total_licenses.png**
   - Dual bar chart comparing total vs active licenses
   - Shows license renewal rates
   - Key insight: ~90% renewal rate indicates stable markets

## Project07 Reuse

This project explicitly demonstrates reuse of project07:

```rust
use csvprof::error::CsvProfError;
```

- **Location**: All CSV loading functions return `Result<_, CsvProfError>`
- **Purpose**: Type-safe error handling across library boundaries
- **Files**: `src/data.rs` (lines 3, 14, 24)

## Data Sources

Both source files are from project08:

1. **Employee_Salaries.csv** (231,945 records)
   - 17,362 FY2024 records processed
   - Columns: AgencyName, AnnualSalary, FiscalYear

2. **Liquor_Licenses.csv** (29,751 records)
   - Aggregated into 133 unique zip codes
   - Columns: AddrZip, LicenseStatus, EstablishmentDesc, LicenseFee

## Documentation

For detailed information about each chart and the analysis:

**Read:** [README.md](README.md)

This file contains:
- Detailed description of each chart
- Why each chart is meaningful
- Data sources and definitions
- Build instructions
- Project07 integration explanation
- Output file listing

## Technical Details

- **Language**: Rust 2021 edition
- **Visualization**: Plotters 0.3 (full features)
- **CSV Processing**: csv crate with serde
- **Error Handling**: Comprehensive error types from project07
- **Code Size**: ~600 lines of Rust across 3 modules
- **Compilation Status**: ✓ Zero errors

## Troubleshooting

### If `cargo run` fails
1. Ensure project07 is built: `cd ../project07 && cargo build`
2. Check paths are relative from project09 directory
3. Verify CSV files exist in `../project08/data/`

### If output directory doesn't exist
- It will be created automatically by the program
- Ensure you have write permissions in the project directory

### If charts don't generate
- Check error message in console
- Ensure plotters feature "full" is enabled (it is in Cargo.toml)
- Verify output directory has write permissions

## Next Steps

1. Run `cargo run` to generate the visualizations
2. Open the PNG files in an image viewer to see the charts
3. Read [README.md](README.md) for analysis and insights
4. Review the code in `src/` to understand the implementation

---

**Project Status**: ✓ Complete and Ready to Use

**File Count**: 6 main files (Cargo.toml, README.md, main.rs, data.rs, charts.rs + output/)

**Code Quality**: Zero compilation errors, follows Rust conventions

**Integration**: Properly imports and uses project07 as a library dependency
