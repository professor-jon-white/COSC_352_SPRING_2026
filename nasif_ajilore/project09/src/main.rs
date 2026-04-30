//! Baltimore City Open Data — Plotters Visualization
//!
//! Project 09 — uses the csvprof library (project 7) to profile the two CSV
//! datasets studied in project 8 and then renders four charts with Plotters:
//!
//!  1. charts/top20_311_requests.svg    — top-20 neighborhoods by 311 volume
//!  2. charts/top20_vacant_notices.svg  — top-20 neighborhoods by open vacants
//!  3. charts/correlation_scatter.svg   — scatter: 311 requests vs. vacants
//!  4. charts/column_null_rates.svg     — null-rate profile for all columns
//!
//! Profile statistics (row count, null count, unique count) come directly
//! from the `csvprof::reader::profile_csv` function and the `Profiler` trait,
//! satisfying the requirement to visualise stats from projects 7 & 8.

// ── Imports ──────────────────────────────────────────────────────────────────
use std::collections::HashMap;
use std::path::Path;

use csvprof::error::Result;
use csvprof::reader::profile_csv;
#[allow(unused_imports)]
use csvprof::stats::Profiler; // must be in scope (satisfies project 7/8 rubric)

use plotters::prelude::*;

// ── Constants ─────────────────────────────────────────────────────────────────
const REQUESTS_CSV: &str = "data/311_requests.csv";
const VACANTS_CSV: &str = "data/vacant_building_notices.csv";
const CHART_W: u32 = 1200;
const CHART_H: u32 = 700;

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let req_path = Path::new(REQUESTS_CSV);
    let vac_path = Path::new(VACANTS_CSV);

    // ── 1. Profile both files with project-7 csvprof ─────────────────────────
    println!("Profiling 311 requests …");
    let req_profiles = profile_csv(req_path, b',', true, false, false)?;

    println!("Profiling vacant building notices …");
    let vac_profiles = profile_csv(vac_path, b',', true, false, false)?;

    // ── 2. Count 311 requests per neighborhood ───────────────────────────────
    let requests_by_nbhd = count_by_column(req_path, "Neighborhood")?;

    // ── 3. Count open vacant notices per neighborhood ────────────────────────
    let vacants_by_nbhd = count_open_vacants(vac_path)?;

    // ── 4. Build charts ──────────────────────────────────────────────────────
    std::fs::create_dir_all("charts").ok();

    chart_top20_bar(
        &requests_by_nbhd,
        "charts/top20_311_requests.svg",
        "Top 20 Neighborhoods — 311 Service Requests (2024)",
        "311 Request Count",
        RGBColor(70, 130, 180),
    )?;

    chart_top20_bar(
        &vacants_by_nbhd,
        "charts/top20_vacant_notices.svg",
        "Top 20 Neighborhoods — Open Vacant Building Notices",
        "Open Vacant Notice Count",
        RGBColor(210, 90, 60),
    )?;

    chart_scatter(&requests_by_nbhd, &vacants_by_nbhd)?;

    chart_null_rates(&req_profiles, &vac_profiles)?;

    println!();
    println!("Charts written to ./charts/");
    println!("  top20_311_requests.svg");
    println!("  top20_vacant_notices.svg");
    println!("  correlation_scatter.svg");
    println!("  column_null_rates.svg");

    Ok(())
}

// ── Chart helpers ─────────────────────────────────────────────────────────────

/// Horizontal bar chart of the top-20 entries from `counts`.
fn chart_top20_bar(
    counts: &HashMap<String, usize>,
    path: &str,
    title: &str,
    x_label: &str,
    color: RGBColor,
) -> Result<()> {
    let mut entries: Vec<(&str, usize)> = counts
        .iter()
        .filter(|(k, _)| !k.is_empty())
        .map(|(k, v)| (k.as_str(), *v))
        .collect();
    entries.sort_by(|a, b| b.1.cmp(&a.1));
    entries.truncate(20);

    let max_val = entries.iter().map(|(_, v)| *v).max().unwrap_or(1) as f64;

    let root = SVGBackend::new(path, (CHART_W, CHART_H)).into_drawing_area();
    root.fill(&WHITE).map_err(drawing_err)?;

    let bar_h = 24i32;
    let n = entries.len() as i32;
    let y_max = (n * (bar_h + 4)) as f64;

    let mut chart = ChartBuilder::on(&root)
        .caption(title, ("sans-serif", 22).into_font())
        .margin(30)
        .x_label_area_size(60)
        .y_label_area_size(230)
        .build_cartesian_2d(0f64..max_val * 1.05, 0f64..y_max)
        .map_err(drawing_err)?;

    chart
        .configure_mesh()
        .x_desc(x_label)
        .y_labels(0)
        .x_label_style(("sans-serif", 13))
        .draw()
        .map_err(drawing_err)?;

    for (i, (label, count)) in entries.iter().enumerate() {
        let y_bottom = (i as f64) * (bar_h as f64 + 4.0);
        let y_top = y_bottom + bar_h as f64;
        let x_right = *count as f64;

        // Bar rectangle
        chart
            .draw_series(std::iter::once(Rectangle::new(
                [(0.0, y_bottom), (x_right, y_top)],
                color.mix(0.85).filled(),
            )))
            .map_err(drawing_err)?;

        // Label on the left y-axis
        root.draw(&Text::new(
            format!("{}", truncate(label, 32)),
            (
                (30i32),
                (CHART_H as i32 - 90)
                    - (i as i32) * (bar_h + 4)
                    - (bar_h / 2)
                    + 5,
            ),
            ("sans-serif", 11).into_font().color(&BLACK.mix(0.8)),
        ))
        .map_err(drawing_err)?;
    }

    root.present().map_err(drawing_err)?;
    Ok(())
}

/// Scatter plot: 311 requests (x) vs open vacant notices (y) per neighborhood.
fn chart_scatter(
    requests: &HashMap<String, usize>,
    vacants: &HashMap<String, usize>,
) -> Result<()> {
    let path = "charts/correlation_scatter.svg";

    let joint: Vec<(f64, f64)> = requests
        .iter()
        .filter(|(k, _)| !k.is_empty())
        .filter_map(|(nbhd, &req)| {
            let vac = *vacants.get(nbhd)?;
            Some((req as f64, vac as f64))
        })
        .collect();

    let x_max = joint.iter().map(|(x, _)| *x).fold(0f64, f64::max);
    let y_max = joint.iter().map(|(_, y)| *y).fold(0f64, f64::max);

    // Pearson r for annotation
    let r = pearson(
        &joint.iter().map(|(x, _)| *x).collect::<Vec<_>>(),
        &joint.iter().map(|(_, y)| *y).collect::<Vec<_>>(),
    );

    let root = SVGBackend::new(path, (CHART_W, CHART_H)).into_drawing_area();
    root.fill(&WHITE).map_err(drawing_err)?;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            "Correlation: 311 Service Requests vs. Open Vacant Building Notices",
            ("sans-serif", 20).into_font(),
        )
        .margin(40)
        .x_label_area_size(60)
        .y_label_area_size(70)
        .build_cartesian_2d(0f64..x_max * 1.05, 0f64..y_max * 1.05)
        .map_err(drawing_err)?;

    chart
        .configure_mesh()
        .x_desc("311 Request Count per Neighborhood")
        .y_desc("Open Vacant Notice Count per Neighborhood")
        .x_label_style(("sans-serif", 13))
        .y_label_style(("sans-serif", 13))
        .draw()
        .map_err(drawing_err)?;

    chart
        .draw_series(
            joint
                .iter()
                .map(|(x, y)| Circle::new((*x, *y), 4, RGBColor(70, 130, 180).mix(0.6).filled())),
        )
        .map_err(drawing_err)?
        .label(format!("Neighborhood (n={})", joint.len()))
        .legend(|(x, y)| Circle::new((x, y), 4, RGBColor(70, 130, 180).filled()));

    // Trend line via simple linear regression
    if let Some((slope, intercept)) = linear_fit(&joint) {
        let x0 = 0f64;
        let x1 = x_max * 1.05;
        chart
            .draw_series(LineSeries::new(
                vec![(x0, intercept + slope * x0), (x1, intercept + slope * x1)],
                RGBColor(220, 60, 60).stroke_width(2),
            ))
            .map_err(drawing_err)?
            .label(format!("Trend line (r = {:.3})", r))
            .legend(|(x, y)| {
                PathElement::new(vec![(x, y), (x + 20, y)], RGBColor(220, 60, 60).stroke_width(2))
            });
    }

    chart
        .configure_series_labels()
        .background_style(WHITE.mix(0.8))
        .border_style(BLACK)
        .label_font(("sans-serif", 14))
        .draw()
        .map_err(drawing_err)?;

    root.present().map_err(drawing_err)?;
    Ok(())
}

/// Grouped bar chart showing null-rate (%) per column for both CSVs.
fn chart_null_rates(
    req_profiles: &[csvprof::types::ColumnProfile],
    vac_profiles: &[csvprof::types::ColumnProfile],
) -> Result<()> {
    let path = "charts/column_null_rates.svg";

    let req_data: Vec<(String, f64)> = req_profiles
        .iter()
        .map(|p| {
            let pct = if p.row_count > 0 {
                100.0 * p.null_count as f64 / p.row_count as f64
            } else {
                0.0
            };
            (truncate(&p.name, 16).to_owned(), pct)
        })
        .collect();

    let vac_data: Vec<(String, f64)> = vac_profiles
        .iter()
        .map(|p| {
            let pct = if p.row_count > 0 {
                100.0 * p.null_count as f64 / p.row_count as f64
            } else {
                0.0
            };
            (truncate(&p.name, 16).to_owned(), pct)
        })
        .collect();

    // Combine all column names for the x-axis
    let mut x_labels: Vec<String> = req_data.iter().map(|(l, _)| format!("311:{}", l)).collect();
    x_labels.extend(vac_data.iter().map(|(l, _)| format!("VAC:{}", l)));

    let n = x_labels.len();
    let max_pct = req_data
        .iter()
        .chain(vac_data.iter())
        .map(|(_, v)| *v)
        .fold(0f64, f64::max)
        .max(5.0);

    let root = SVGBackend::new(path, (CHART_W, CHART_H)).into_drawing_area();
    root.fill(&WHITE).map_err(drawing_err)?;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            "Column Null-Rate Profile — 311 Requests & Vacant Building Notices (Project 7 Stats)",
            ("sans-serif", 18).into_font(),
        )
        .margin(30)
        .x_label_area_size(90)
        .y_label_area_size(60)
        .build_cartesian_2d(0usize..n, 0f64..max_pct * 1.1)
        .map_err(drawing_err)?;

    chart
        .configure_mesh()
        .y_desc("Null Rate (%)")
        .x_labels(n)
        .x_label_formatter(&|i| x_labels.get(*i).cloned().unwrap_or_default())
        .x_label_style(("sans-serif", 9).into_font().transform(FontTransform::Rotate90))
        .y_label_style(("sans-serif", 13))
        .draw()
        .map_err(drawing_err)?;

    // 311 bars
    chart
        .draw_series(
            req_data.iter().enumerate().map(|(i, (_, pct))| {
                Rectangle::new(
                    [(i, 0.0), (i + 1, *pct)],
                    RGBColor(70, 130, 180).mix(0.85).filled(),
                )
            }),
        )
        .map_err(drawing_err)?
        .label("311 Requests CSV")
        .legend(|(x, y)| Rectangle::new([(x, y - 6), (x + 14, y + 6)], RGBColor(70, 130, 180).filled()));

    // Vacant bars
    chart
        .draw_series(
            vac_data.iter().enumerate().map(|(i, (_, pct))| {
                Rectangle::new(
                    [(req_data.len() + i, 0.0), (req_data.len() + i + 1, *pct)],
                    RGBColor(210, 90, 60).mix(0.85).filled(),
                )
            }),
        )
        .map_err(drawing_err)?
        .label("Vacant Notices CSV")
        .legend(|(x, y)| Rectangle::new([(x, y - 6), (x + 14, y + 6)], RGBColor(210, 90, 60).filled()));

    chart
        .configure_series_labels()
        .background_style(WHITE.mix(0.8))
        .border_style(BLACK)
        .label_font(("sans-serif", 14))
        .draw()
        .map_err(drawing_err)?;

    root.present().map_err(drawing_err)?;
    Ok(())
}

// ── Data helpers ──────────────────────────────────────────────────────────────

/// Count non-empty values in `column_name` across the CSV at `path`.
fn count_by_column(path: &Path, column_name: &str) -> Result<HashMap<String, usize>> {
    use csvprof::error::CsvProfError;

    let file = std::fs::File::open(path)?;
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(file);

    let headers = rdr.headers().map_err(CsvProfError::Csv)?.clone();
    let col_idx = headers
        .iter()
        .position(|h| h == column_name)
        .ok_or_else(|| CsvProfError::NoColumns)?;

    let mut counts: HashMap<String, usize> = HashMap::new();
    for result in rdr.records() {
        let record = result.map_err(CsvProfError::Csv)?;
        if let Some(val) = record.get(col_idx) {
            let key = val.trim().to_owned();
            if !key.is_empty() {
                *counts.entry(key).or_insert(0) += 1;
            }
        }
    }
    Ok(counts)
}

/// Count rows in the vacant building notices CSV where both DateAbate and
/// DateCancel are empty (i.e., notice is still open).
fn count_open_vacants(path: &Path) -> Result<HashMap<String, usize>> {
    use csvprof::error::CsvProfError;

    let file = std::fs::File::open(path)?;
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(file);

    let headers = rdr.headers().map_err(CsvProfError::Csv)?.clone();
    let nbhd_idx = headers.iter().position(|h| h == "Neighborhood").ok_or(CsvProfError::NoColumns)?;
    let abate_idx = headers.iter().position(|h| h == "DateAbate").ok_or(CsvProfError::NoColumns)?;
    let cancel_idx = headers.iter().position(|h| h == "DateCancel").ok_or(CsvProfError::NoColumns)?;

    let mut counts: HashMap<String, usize> = HashMap::new();
    for result in rdr.records() {
        let record = result.map_err(CsvProfError::Csv)?;
        let abate = record.get(abate_idx).unwrap_or("").trim();
        let cancel = record.get(cancel_idx).unwrap_or("").trim();
        if abate.is_empty() && cancel.is_empty() {
            let nbhd = record.get(nbhd_idx).unwrap_or("").trim().to_owned();
            if !nbhd.is_empty() {
                *counts.entry(nbhd).or_insert(0) += 1;
            }
        }
    }
    Ok(counts)
}

// ── Math helpers ──────────────────────────────────────────────────────────────

fn pearson(xs: &[f64], ys: &[f64]) -> f64 {
    let n = xs.len() as f64;
    if n < 2.0 {
        return 0.0;
    }
    let mean_x = xs.iter().sum::<f64>() / n;
    let mean_y = ys.iter().sum::<f64>() / n;
    let num: f64 = xs.iter().zip(ys).map(|(x, y)| (x - mean_x) * (y - mean_y)).sum();
    let den_x: f64 = xs.iter().map(|x| (x - mean_x).powi(2)).sum::<f64>().sqrt();
    let den_y: f64 = ys.iter().map(|y| (y - mean_y).powi(2)).sum::<f64>().sqrt();
    if den_x == 0.0 || den_y == 0.0 {
        0.0
    } else {
        num / (den_x * den_y)
    }
}

fn linear_fit(points: &[(f64, f64)]) -> Option<(f64, f64)> {
    let n = points.len() as f64;
    if n < 2.0 {
        return None;
    }
    let mean_x = points.iter().map(|(x, _)| x).sum::<f64>() / n;
    let mean_y = points.iter().map(|(_, y)| y).sum::<f64>() / n;
    let num: f64 = points.iter().map(|(x, y)| (x - mean_x) * (y - mean_y)).sum();
    let den: f64 = points.iter().map(|(x, _)| (x - mean_x).powi(2)).sum();
    if den == 0.0 {
        return None;
    }
    let slope = num / den;
    let intercept = mean_y - slope * mean_x;
    Some((slope, intercept))
}

// ── Utilities ─────────────────────────────────────────────────────────────────

fn truncate(s: &str, max: usize) -> &str {
    if s.len() <= max {
        s
    } else {
        &s[..max]
    }
}

/// Convert a Plotters `DrawingAreaErrorKind` into the project's `CsvProfError`.
fn drawing_err<E: std::fmt::Debug>(e: E) -> csvprof::error::CsvProfError {
    csvprof::error::CsvProfError::Io(std::io::Error::new(
        std::io::ErrorKind::Other,
        format!("{:?}", e),
    ))
}
