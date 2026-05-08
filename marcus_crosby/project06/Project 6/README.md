# Baltimore City Homicide Analysis Dashboard

This project is a Dockerized R Shiny dashboard for exploring Baltimore City homicide data from Chams Page's 2025 homicide list.

The app is designed for operational analysis: commanders can filter incidents, review clearance rates, inspect victim age patterns, compare methods by month, and view incident locations on an interactive map.

## Project Files

- `scrape_data.R` scrapes the homicide table from Chams Page, cleans the messy HTML table, derives analysis fields, geocodes addresses, and writes `homicide_data.csv`.
- `app.R` loads `homicide_data.csv` and serves the interactive Shiny dashboard.
- `Dockerfile` builds an R container with Shiny, scraping, plotting, table, and mapping dependencies.
- `run_dashboard.sh` scrapes data if needed, builds the Docker image, and runs the dashboard on port `3838`.

## Data Source

The scraper reads from:

`https://chamspage.blogspot.com/2025/01/2025-baltimore-city-homicide-list.html`

The generated dataset includes fields such as victim name, age, death date, address, method, nearby CCTV camera count, case closure status, and map coordinates where geocoding succeeds.

## Quick Start With Docker

Make sure Docker is installed and running, then run:

```bash
./run_dashboard.sh
```

Open the dashboard at:

```text
http://localhost:3838
```

The script creates `homicide_data.csv` automatically if it is missing.

## Run Locally Without Docker

Install the required R packages:

```r
install.packages(c(
  "shiny", "rvest", "xml2", "dplyr", "stringr", "lubridate",
  "jsonlite", "ggplot2", "plotly", "leaflet", "DT", "bslib",
  "scales", "htmltools"
))
```

Scrape and prepare the data:

```bash
Rscript scrape_data.R
```

Start the app:

```bash
Rscript -e "shiny::runApp('.', host = '127.0.0.1', port = 3838)"
```

## Dashboard Features

- Date range, victim age, method, counted-case, closed-case, and CCTV filters.
- Summary metrics for filtered homicide count, clearance rate, average victim age, and CCTV coverage.
- Interactive Plotly histogram of victim ages.
- Interactive Plotly monthly method chart.
- Leaflet map of geocoded incident locations.
- Searchable and filterable case table.

## Notes

Geocoding uses the U.S. Census geocoder. A small number of block-level addresses may not return coordinates, but those records still remain available in the charts, summary statistics, and table.
