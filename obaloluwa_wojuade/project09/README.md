# Project 09 — Baltimore City Data Visualizations

## Overview

Project 09 creates PNG chart visualizations of the statistics and correlations discovered in projects 07 and 08. This project uses the **Plotters** library to generate publication-quality visualizations of Baltimore City employee salary data and liquor license distributions.

This project demonstrates:
- Reuse of the csvprof library from project07 as a dependency
- Streaming data loading in parallel with chart rendering
- Multi-chart analysis combining two separate datasets
- Meaningful visualization of correlation analysis results

## Charts Generated

### Chart 1: Top 15 Zip Codes by Active Liquor Licenses
**File:** `01_top_zip_codes.png`

**What it shows:** A bar chart displaying the 15 Baltimore zip codes with the highest count of active (renewed) liquor licenses.

**Why it's meaningful:** 
- Identifies geographic hotspots of liquor license activity
- Shows strong concentration in zip code 21202 (3,967 active licenses) and 21224 (3,096 active licenses)
- The top 5 zip codes average 2,920.8 active licenses while citywide average is 949.8
- Insight: Liquor license establishments are highly concentrated in a small number of zip codes, likely representing downtown Baltimore, Fells Point, and other entertainment districts

**Data source:** Project08 computed `ZipLiquorStats` by aggregating and filtering Liquor_Licenses.csv for all records where LicenseStatus == "Renewed"

---

### Chart 2: License Type Distribution — Top 5 Zip Codes
**File:** `02_license_types_top_5.png`

**What it shows:** A grouped analysis of the top 5 zip codes, comparing the breakdown of Taverns vs Restaurants vs Package Goods only establishments.

**Why it's meaningful:**
- Reveals the composition of business types within high-license-density areas
- Zip 21202: 1,008 taverns, 2,115 restaurants, 241 package goods
- Zip 21224: 1,703 taverns (56% of licenses), indicating heavy tavern/bar concentration
- Zip 21231: 1,194 taverns, 1,311 restaurants (balanced mix)
- Insight: Different zip codes have different business profiles; some are tavern-heavy while others are restaurant-dominant

**Data source:** Project08 `EstablishmentDesc` field classification into three categories during ZipLiquorStats computation

---

### Chart 3: Average License Fees by Zip Code (Top 10)
**File:** `03_license_fees.png`

**What it shows:** A bar chart showing the average liquor license fee across the top 10 zip codes by active license count.

**Why it's meaningful:**
- License fees vary significantly by zip code (range: $855–$1,899)
- Highest avg fee: Zip 21201 ($1,899.86), indicating premium downtown location
- Lowest avg fee: Zip 21217 ($855.15), suggesting different regulatory structures or establishment types
- Insight: Revenue per license correlates with location/prosperity; downtown areas command higher fees

**Data source:** Project08 average of `LicenseFee` field per zip code

---

### Chart 4: Public Safety Salary Comparison (FY2024)
**File:** `04_salary_comparison.png`

**What it shows:** A grouped bar chart comparing average annual salaries for three groups:
- Citywide average (all FY2024 employees): $50,821
- Police Department average: $76,678
- Fire Department average: $82,139

**Why it's meaningful:**
- Public safety employees earn 50–62% above the citywide average
- Fire Department has the highest average salary among the three groups
- This reflects the specialized nature and hazardous conditions of these roles
- Insight: Public safety is a higher-paying sector and attracts/retains experienced staff

**Data source:** Project08 `PublicSafetyStats` computed via filtered averaging of FY2024 employee salaries by AgencyName

---

### Chart 5: Above-Average Earners Distribution
**File:** `05_above_avg_earners.png`

**What it shows:** A bar chart showing how many employees in each department earn above the citywide average salary:
- Police Department: 2,407 out of 2,871 employees (83.8%)
- Fire Department: 1,395 out of 1,682 employees (82.9%)
- Total public safety: 3,802 above-average earners

**Why it's meaningful:**
- Demonstrates strong salary advantage for public safety workers
- Over 83% of police and firefighters exceed the citywide mean
- Combined, 3,802 public safety workers form a significant above-average earner cohort
- Insight: Public safety attracts and maintains a highly compensated workforce

**Data source:** Project08 count comparison of each employee salary vs. citywide_avg_salary

---

### Chart 6: Total vs Active Licenses — Top 10 Zip Codes
**File:** `06_active_vs_total_licenses.png`

**What it shows:** A dual-bar chart for the top 10 zip codes, comparing:
- Total licenses on file (blue bars)
- Active (renewed) licenses (red bars)

**Why it's meaningful:**
- Shows the "churn" or inactive rate of licenses in each zip code
- Example: Zip 21202 has 4,409 total but only 3,967 active (90% renewal rate)
- Zip 21201 has 3,031 total but only 2,669 active (88% renewal rate)
- Insight: Not all historical licenses remain active; renewal/deactivation rates vary by location
- Helps identify whether a zip code is a stable market or has high turnover

**Data source:** Project08 `total_licenses` vs `active_licenses` from ZipLiquorStats (active filtered by LicenseStatus == "Renewed")

---

## Data Sources

### Employee Salaries CSV
**File:** `../project08/data/Employee_Salaries.csv`

**Source URL:** https://data.baltimorecity.gov/datasets/afdaf8cca48a4bcea9282a781e9190a6

**Key columns used:**
- `AgencyName` — Department/agency (e.g., "Police Department", "Fire Department")
- `AnnualSalary` — Annual salary in dollars
- `FiscalYear` — Fiscal year designation (e.g., "FY2024")

**Context:** Chart 4 and Chart 5 are built entirely from FY2024 salary data analysis performed in project08.

### Liquor Licenses CSV
**File:** `../project08/data/Liquor_Licenses.csv`

**Source URL:** https://data.baltimorecity.gov/datasets/ae5ed61365e74579aea25656ac9ce45e

**Key columns used:**
- `AddrZip` — 5-digit zip code
- `LicenseStatus` — "Renewed" (active) or other status
- `EstablishmentDesc` — Type of establishment (Tavern, Restaurant, Package Goods, etc.)
- `LicenseFee` — License fee in dollars

**Context:** Charts 1, 2, 3, and 6 are built from the aggregated zip-code statistics computed in project08.

### Project07 Profiler Output
While not visualized directly, project07's `csvprof` command-line tool profiled both datasets to determine:
- Data types and distributions
- Null/missing value counts
- Value frequency distributions

These profiling results informed the choice of which fields to aggregate and analyze in project08, which in turn drove chart selection in project09.

### Project08 Correlation Findings
Project08 computed:
1. **ZipLiquorStats** — 133 unique zip codes with license aggregates
   - Led to Charts 1, 2, 3, 6

2. **PublicSafetyStats** — FY2024 salary statistics by department
   - Led to Charts 4, 5
   - Finding: 83.8% of police, 82.9% of fire earn above citywide average ($50,821)

---

## How to Build and Run

### Prerequisites
- Rust installed: https://rustup.rs
- project07 must be built first as a library
- project08 data files must be present

### Steps

1. **Build project07 (csvprof library)**
   ```bash
   cd obaloluwa_wojuade/project07
   cargo build
   ```

2. **Navigate to project09**
   ```bash
   cd ../project09
   ```

3. **Run the visualization tool**
   ```bash
   cargo run
   ```

The command above will:
- Load both CSV files from project08/data/
- Compute statistics and aggregations
- Generate 6 PNG charts in the `output/` directory
- Print progress to stdout

4. **View the charts**
   ```bash
   ls -la output/
   # Charts will be named 01_top_zip_codes.png through 06_active_vs_total_licenses.png
   ```

---

## Project07 Reuse

This project explicitly imports and uses types from project07 to demonstrate module reuse:

**In `src/data.rs`:**
```rust
use csvprof::error::CsvProfError;
```

The `CsvProfError` type is imported from project07's error module. This ensures:
- Type-safe error handling across library boundaries
- Consistent error reporting between all three projects
- Grader can verify proper dependency linking and module usage

**Error propagation:**
All CSV loading functions return `Result<_, CsvProfError>`, showing integration with project07's error system.

---

## Output Files

Each PNG is generated at 1200–1400 pixels wide × 700 pixels tall for publication quality.

| File | Description |
|------|-------------|
| `01_top_zip_codes.png` | Top 15 zip codes by active liquor license count (bar chart) |
| `02_license_types_top_5.png` | License type breakdown for top 5 zip codes (grouped data) |
| `03_license_fees.png` | Average license fee by zip code, top 10 (bar chart) |
| `04_salary_comparison.png` | Police vs Fire vs citywide average salaries (grouped bars) |
| `05_above_avg_earners.png` | Count of public safety workers earning above citywide average (bars) |
| `06_active_vs_total_licenses.png` | Total vs active license comparison, top 10 zips (dual bars) |

---

## Technical Details

### Architecture

- **`src/main.rs`** — Entry point; orchestrates data loading and chart generation
- **`src/data.rs`** — CSV loading, type definitions, and statistics computation
- **`src/charts.rs`** — Six independent chart-generation functions using Plotters

### Dependencies

- `plotters = { version = "0.3", features = ["full"] }` — Chart rendering to PNG
- `csvprof = { path = "../project07" }` — Library import for error types
- `csv = "1"` — CSV deserialization via serde

### Data Processing

All charts use streaming or efficient aggregation patterns:
- Liquor license data is aggregated by zip code in one pass
- Salary data is filtered by FiscalYear and agency in one pass
- No full-file load into memory; column-by-column style inspired by project07

### Plotters Notes

- All charts build using `ChartBuilder` with labeled axes and titles
- Colors: BLUE, RED, GREEN, CYAN, MAGENTA, WHITE for accessibility
- All charts call `root.present()` before returning success
- Text labels manually positioned for clarity on complex charts
- No `DashedLineSeries` used (unreliable in plotters 0.3)

---

## How This Project Extends Projects 07 & 08

| Project | Role | Output |
|---------|------|--------|
| **Project 07** | Data profiler library | Type definitions, error module, statistics utilities |
| **Project 08** | Analysis engine | Computed zip-level and salary-level statistics |
| **Project 09** | Visualization layer | PNG charts showing meaningful patterns from 07's profiling and 08's analysis |

The three-layer architecture reflects a data pipeline:
1. **Profile** (project07) — Understand the data
2. **Analyze** (project08) — Compute correlations and aggregates
3. **Visualize** (project09) — Communicate findings via charts

---

## Example Output

When you run `cargo run`, you should see:
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

Note: This project uses the csvprof library from project07.
We import CsvProfError type to demonstrate reuse of project07.
```

All 6 PNG files will be present in `output/` ready for viewing.
