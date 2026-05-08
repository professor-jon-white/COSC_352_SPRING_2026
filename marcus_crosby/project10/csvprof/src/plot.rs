use std::fs::File;
use std::io::{BufReader, Read};
use std::ops::Range;
use std::path::Path;

use anyhow::{anyhow, bail, ensure, Context};
use clap::ValueEnum;
use csv::{ReaderBuilder, StringRecord};
use plotters::coord::types::RangedCoordf64;
use plotters::coord::Shift;
use plotters::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum PlotKind {
    Scatter,
    Line,
    Bar,
}

#[derive(Debug, Clone)]
pub struct CsvPlotConfig {
    pub delimiter: u8,
    pub has_headers: bool,
    pub x_column: String,
    pub y_column: String,
    pub kind: PlotKind,
    pub limit: usize,
    pub top: usize,
}

#[derive(Debug, Clone)]
pub struct PlotRenderConfig {
    pub width: u32,
    pub height: u32,
    pub title: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CsvPlotDataset {
    pub x_label: String,
    pub y_label: String,
    pub kind: PlotKind,
    pub rows: Vec<CsvPlotRow>,
    pub skipped_rows: u64,
}

#[derive(Debug, Clone)]
pub struct CsvPlotRow {
    pub x_label: String,
    pub x: f64,
    pub y: f64,
}

pub fn load_csv_plot_file(path: &Path, config: &CsvPlotConfig) -> anyhow::Result<CsvPlotDataset> {
    let file = File::open(path).with_context(|| format!("failed to open `{}`", path.display()))?;
    load_csv_plot_data(BufReader::new(file), config)
}

pub fn load_csv_plot_data<R: Read>(
    reader: R,
    config: &CsvPlotConfig,
) -> anyhow::Result<CsvPlotDataset> {
    ensure!(config.limit > 0, "plot row limit must be greater than zero");
    ensure!(
        config.top > 0,
        "bar chart top count must be greater than zero"
    );

    let mut csv_reader = ReaderBuilder::new()
        .delimiter(config.delimiter)
        .has_headers(config.has_headers)
        .flexible(true)
        .trim(csv::Trim::All)
        .from_reader(reader);

    let headers = if config.has_headers {
        Some(
            csv_reader
                .headers()
                .context("failed to read CSV headers")?
                .clone(),
        )
    } else {
        None
    };

    let (x_index, x_label) = resolve_column(headers.as_ref(), &config.x_column, "x")?;
    let (y_index, y_label) = resolve_column(headers.as_ref(), &config.y_column, "y")?;

    let mut rows = Vec::new();
    let mut skipped_rows = 0;

    for record in csv_reader.records() {
        let record = record.context("failed to read CSV record")?;
        if rows.len() >= config.limit {
            break;
        }

        match read_plot_row(&record, x_index, y_index, config.kind, rows.len()) {
            Some(row) => rows.push(row),
            None => skipped_rows += 1,
        }
    }

    if config.kind == PlotKind::Bar {
        rows.sort_by(|left, right| {
            right
                .y
                .partial_cmp(&left.y)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| left.x_label.cmp(&right.x_label))
        });
        rows.truncate(config.top);
    }

    ensure!(
        !rows.is_empty(),
        "no plottable rows found for `{}` and `{}`",
        config.x_column,
        config.y_column
    );

    Ok(CsvPlotDataset {
        x_label,
        y_label,
        kind: config.kind,
        rows,
        skipped_rows,
    })
}

pub fn render_csv_plot(
    dataset: &CsvPlotDataset,
    path: &Path,
    config: &PlotRenderConfig,
) -> anyhow::Result<()> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create `{}`", parent.display()))?;
    }

    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    if extension == "svg" {
        let root = SVGBackend::new(path, (config.width, config.height)).into_drawing_area();
        draw_plot(root, dataset, config)
    } else {
        let root = BitMapBackend::new(path, (config.width, config.height)).into_drawing_area();
        draw_plot(root, dataset, config)
    }
}

fn read_plot_row(
    record: &StringRecord,
    x_index: usize,
    y_index: usize,
    kind: PlotKind,
    row_index: usize,
) -> Option<CsvPlotRow> {
    let x_raw = record.get(x_index)?.trim();
    let y = parse_number(record.get(y_index)?)?;

    match kind {
        PlotKind::Bar => {
            if x_raw.is_empty() {
                return None;
            }
            Some(CsvPlotRow {
                x_label: x_raw.to_string(),
                x: row_index as f64,
                y,
            })
        }
        PlotKind::Scatter | PlotKind::Line => {
            let x = parse_number(x_raw)?;
            Some(CsvPlotRow {
                x_label: x_raw.to_string(),
                x,
                y,
            })
        }
    }
}

fn draw_plot<DB>(
    root: DrawingArea<DB, Shift>,
    dataset: &CsvPlotDataset,
    config: &PlotRenderConfig,
) -> anyhow::Result<()>
where
    DB: DrawingBackend,
{
    root.fill(&WHITE)
        .map_err(|err| anyhow!("failed to clear plot canvas: {err:?}"))?;

    match dataset.kind {
        PlotKind::Scatter => draw_scatter(&root, dataset, config)?,
        PlotKind::Line => draw_line(&root, dataset, config)?,
        PlotKind::Bar => draw_bar(&root, dataset, config)?,
    }

    root.present()
        .map_err(|err| anyhow!("failed to write plot: {err:?}"))
}

fn draw_scatter<DB>(
    root: &DrawingArea<DB, Shift>,
    dataset: &CsvPlotDataset,
    config: &PlotRenderConfig,
) -> anyhow::Result<()>
where
    DB: DrawingBackend,
{
    let x_range = numeric_range(dataset.rows.iter().map(|row| row.x))?;
    let y_range = numeric_range(dataset.rows.iter().map(|row| row.y))?;
    let mut chart = build_numeric_chart(root, dataset, config, x_range, y_range)?;

    chart
        .draw_series(
            dataset
                .rows
                .iter()
                .map(|row| Circle::new((row.x, row.y), 4, RGBColor(37, 99, 235).filled())),
        )
        .map_err(|err| anyhow!("failed to draw scatter points: {err:?}"))?;

    Ok(())
}

fn draw_line<DB>(
    root: &DrawingArea<DB, Shift>,
    dataset: &CsvPlotDataset,
    config: &PlotRenderConfig,
) -> anyhow::Result<()>
where
    DB: DrawingBackend,
{
    let x_range = numeric_range(dataset.rows.iter().map(|row| row.x))?;
    let y_range = numeric_range(dataset.rows.iter().map(|row| row.y))?;
    let mut chart = build_numeric_chart(root, dataset, config, x_range, y_range)?;

    chart
        .draw_series(LineSeries::new(
            dataset.rows.iter().map(|row| (row.x, row.y)),
            RGBColor(37, 99, 235).stroke_width(3),
        ))
        .map_err(|err| anyhow!("failed to draw line series: {err:?}"))?;
    chart
        .draw_series(
            dataset
                .rows
                .iter()
                .map(|row| Circle::new((row.x, row.y), 3, RGBColor(15, 23, 42).filled())),
        )
        .map_err(|err| anyhow!("failed to draw line markers: {err:?}"))?;

    Ok(())
}

fn draw_bar<DB>(
    root: &DrawingArea<DB, Shift>,
    dataset: &CsvPlotDataset,
    config: &PlotRenderConfig,
) -> anyhow::Result<()>
where
    DB: DrawingBackend,
{
    let count = i32::try_from(dataset.rows.len()).context("too many bars to render")?;
    let y_range = numeric_range_with_zero(dataset.rows.iter().map(|row| row.y))?;
    let caption = plot_title(dataset, config);
    let x_label_area_size = if dataset.rows.len() > 8 { 220 } else { 128 };

    let mut chart = ChartBuilder::on(root)
        .caption(caption, ("sans-serif", 28))
        .margin(24)
        .x_label_area_size(x_label_area_size)
        .y_label_area_size(72)
        .build_cartesian_2d(0..count, y_range)
        .map_err(|err| anyhow!("failed to build bar chart: {err:?}"))?;

    chart
        .configure_mesh()
        .disable_mesh()
        .x_desc(&dataset.x_label)
        .y_desc(&dataset.y_label)
        .x_labels(dataset.rows.len().min(20))
        .axis_desc_style(("sans-serif", 14).into_font())
        .x_label_style(
            TextStyle::from(("sans-serif", 14).into_font()).transform(FontTransform::Rotate90),
        )
        .x_label_formatter(&|index| {
            let index = (*index).clamp(0, count.saturating_sub(1)) as usize;
            truncate_label(&dataset.rows[index].x_label, 32)
        })
        .draw()
        .map_err(|err| anyhow!("failed to draw bar chart mesh: {err:?}"))?;

    chart
        .draw_series(dataset.rows.iter().enumerate().map(|(index, row)| {
            let index = index as i32;
            Rectangle::new(
                [(index, 0.0), (index + 1, row.y)],
                RGBColor(37, 99, 235).filled(),
            )
        }))
        .map_err(|err| anyhow!("failed to draw bars: {err:?}"))?;

    Ok(())
}

fn build_numeric_chart<'a, DB>(
    root: &'a DrawingArea<DB, Shift>,
    dataset: &CsvPlotDataset,
    config: &PlotRenderConfig,
    x_range: Range<f64>,
    y_range: Range<f64>,
) -> anyhow::Result<ChartContext<'a, DB, Cartesian2d<RangedCoordf64, RangedCoordf64>>>
where
    DB: DrawingBackend,
{
    let caption = plot_title(dataset, config);
    let mut chart = ChartBuilder::on(root)
        .caption(caption, ("sans-serif", 28))
        .margin(24)
        .x_label_area_size(56)
        .y_label_area_size(72)
        .build_cartesian_2d(x_range, y_range)
        .map_err(|err| anyhow!("failed to build numeric chart: {err:?}"))?;

    chart
        .configure_mesh()
        .light_line_style(RGBColor(226, 232, 240))
        .bold_line_style(RGBColor(148, 163, 184))
        .x_desc(&dataset.x_label)
        .y_desc(&dataset.y_label)
        .draw()
        .map_err(|err| anyhow!("failed to draw chart mesh: {err:?}"))?;

    Ok(chart)
}

fn resolve_column(
    headers: Option<&StringRecord>,
    selector: &str,
    role: &str,
) -> anyhow::Result<(usize, String)> {
    if let Some(headers) = headers {
        if let Some((index, header)) = headers
            .iter()
            .enumerate()
            .find(|(_, header)| clean_header(header).eq_ignore_ascii_case(selector))
        {
            return Ok((index, clean_header(header).to_string()));
        }
    }

    let selector = selector.trim();
    let index = selector
        .strip_prefix("column_")
        .unwrap_or(selector)
        .parse::<usize>()
        .with_context(|| {
            let available = headers
                .map(|headers| {
                    headers
                        .iter()
                        .map(clean_header)
                        .collect::<Vec<_>>()
                        .join(", ")
                })
                .unwrap_or_else(|| "1-based column indexes".to_string());
            format!("could not resolve {role} column `{selector}`; available: {available}")
        })?;

    if index == 0 {
        bail!("{role} column indexes are 1-based; got 0");
    }
    let zero_based = index - 1;
    if let Some(headers) = headers {
        let header = headers
            .get(zero_based)
            .with_context(|| format!("{role} column index {index} is out of range"))?;
        return Ok((zero_based, clean_header(header).to_string()));
    }

    Ok((zero_based, format!("column_{index}")))
}

fn parse_number(raw: &str) -> Option<f64> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }

    let normalized = trimmed.replace(',', "");
    normalized
        .parse::<f64>()
        .ok()
        .filter(|value| value.is_finite())
}

fn numeric_range<I>(values: I) -> anyhow::Result<Range<f64>>
where
    I: IntoIterator<Item = f64>,
{
    let (min, max) = min_max(values)?;
    Ok(padded_range(min, max))
}

fn numeric_range_with_zero<I>(values: I) -> anyhow::Result<Range<f64>>
where
    I: IntoIterator<Item = f64>,
{
    let (min, max) = min_max(values)?;
    Ok(padded_range(min.min(0.0), max.max(0.0)))
}

fn min_max<I>(values: I) -> anyhow::Result<(f64, f64)>
where
    I: IntoIterator<Item = f64>,
{
    let mut values = values.into_iter();
    let first = values
        .next()
        .ok_or_else(|| anyhow!("no numeric values available to plot"))?;
    let mut min = first;
    let mut max = first;
    for value in values {
        min = min.min(value);
        max = max.max(value);
    }
    Ok((min, max))
}

fn padded_range(min: f64, max: f64) -> Range<f64> {
    if (max - min).abs() < f64::EPSILON {
        let pad = if min.abs() < 1.0 {
            1.0
        } else {
            min.abs() * 0.10
        };
        return (min - pad)..(max + pad);
    }

    let pad = (max - min).abs() * 0.08;
    (min - pad)..(max + pad)
}

fn plot_title(dataset: &CsvPlotDataset, config: &PlotRenderConfig) -> String {
    config.title.clone().unwrap_or_else(|| {
        let kind = match dataset.kind {
            PlotKind::Scatter => "Scatter",
            PlotKind::Line => "Line",
            PlotKind::Bar => "Bar",
        };
        format!("{kind}: {} vs {}", dataset.y_label, dataset.x_label)
    })
}

fn truncate_label(value: &str, limit: usize) -> String {
    let chars = value.chars().take(limit).collect::<String>();
    if value.chars().count() > limit {
        format!("{chars}...")
    } else {
        chars
    }
}

fn clean_header(header: &str) -> &str {
    header.trim_start_matches('\u{feff}').trim()
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    fn base_config(kind: PlotKind) -> CsvPlotConfig {
        CsvPlotConfig {
            delimiter: b',',
            has_headers: true,
            x_column: "x".to_string(),
            y_column: "y".to_string(),
            kind,
            limit: 100,
            top: 10,
        }
    }

    #[test]
    fn loads_scatter_points_by_header_and_skips_bad_rows() {
        let csv = "name,x,y\nA,1,2\nB,,4\nC,2,nope\nD,3,4\n";
        let dataset = load_csv_plot_data(Cursor::new(csv), &base_config(PlotKind::Scatter))
            .expect("plot data should load");

        assert_eq!(dataset.rows.len(), 2);
        assert_eq!(dataset.skipped_rows, 2);
        assert_eq!(dataset.rows[0].x, 1.0);
        assert_eq!(dataset.rows[1].y, 4.0);
    }

    #[test]
    fn bar_plots_keep_largest_requested_labels() {
        let csv = "x,y\nNorth,3\nSouth,9\nEast,5\n";
        let mut config = base_config(PlotKind::Bar);
        config.top = 2;
        let dataset =
            load_csv_plot_data(Cursor::new(csv), &config).expect("bar plot data should load");

        assert_eq!(
            dataset
                .rows
                .iter()
                .map(|row| row.x_label.as_str())
                .collect::<Vec<_>>(),
            vec!["South", "East"]
        );
    }

    #[test]
    fn renders_svg_plot() {
        let csv = "x,y\n1,2\n2,4\n3,8\n";
        let dataset = load_csv_plot_data(Cursor::new(csv), &base_config(PlotKind::Line))
            .expect("plot data should load");
        let path = std::env::temp_dir().join(format!(
            "csvprof-plot-test-{}-{}.svg",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time should be valid")
                .as_nanos()
        ));

        render_csv_plot(
            &dataset,
            &path,
            &PlotRenderConfig {
                width: 640,
                height: 360,
                title: Some("Test Plot".to_string()),
            },
        )
        .expect("plot should render");
        let rendered = std::fs::read_to_string(&path).expect("rendered SVG should be readable");
        std::fs::remove_file(&path).ok();

        assert!(rendered.contains("<svg"));
        assert!(rendered.contains("Test Plot"));
    }
}
