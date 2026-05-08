# Baltimore City Homicide Age Analysis

This project scrapes Baltimore City homicide-list tables from Chamspage and builds a reproducible R analysis inside Docker.

Primary source:
https://chamspage.blogspot.com/2025/01/2025-baltimore-city-homicide-list.html

The R script also queries the blog's Blogger/Atom feed and, when available, pulls annual Baltimore City homicide-list posts from 2015 through 2026. The cleaned dataset and chart are written to `output/` when `run.sh` is used.

## Statistic

The histogram shows the distribution of victim ages in 5-year bins, colored by method inferred from the notes field. Age is a useful statistic because it reveals concentration by life stage across many years, while the method color adds context without reducing the analysis to a simple total count.

The script prints the histogram bin counts to stdout so the result is visible even when running in Docker.

## Cleaning Decisions

- The scraper identifies the homicide table by header text instead of assuming it is always the first table, because some pages contain maps or nested tables.
- Rows are parsed from direct table cells only, which keeps nested note tables from becoming separate homicide records.
- If malformed HTML creates extra cells, the first five fields and last three fields are preserved, and the middle cells are collapsed into `notes`.
- Non-numbered rows such as `XXX` and rows with missing or implausible ages are retained in the cleaned CSV but excluded from the age histogram.
- Method is inferred from note keywords such as shooting, stabbing, trauma, vehicle, and fire. Ambiguous rows are labeled `Other/unknown`.
- Dates are parsed from the first recognizable date-like value in `Date Died`; partial year/month values are kept as best-effort dates for structured output.

## Run

```bash
./run.sh
```

The script builds the Docker image `homicide-analysis`, runs it, streams the analysis output to the terminal, and saves:

- `output/baltimore_homicides_cleaned.csv`
- `output/baltimore_homicide_age_histogram.png`
- `output/run.log`
