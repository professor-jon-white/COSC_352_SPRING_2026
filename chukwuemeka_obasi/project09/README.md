# CSV Visualization with Plotters

This project extends the Rust CSV profiling work by rendering two PNG dashboards:

- `profile_dashboard.png` shows profile-oriented statistics such as null counts, numeric ranges, means, and categorical cardinality.
- `correlation_dashboard.png` shows a Pearson correlation heatmap and a scatter plot for the strongest numeric relationship in the file.

## Run

```bash
cargo run -- path/to/data.csv --out-dir output
```

The generated images will be written into the output directory you choose.

## Project 8 Data

These commands render dashboards for the Baltimore water-quality CSVs already used in `project08`:

```bash
cargo run -- ../project08/data/Surface_Water_Quality_Data_2006_to_2015.csv --out-dir output/surface_water_2006_2015
cargo run -- ../project08/data/Surface_Water_Quality_Data_2016_2025.csv --out-dir output/surface_water_2016_2025
```
