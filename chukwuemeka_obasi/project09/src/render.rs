use std::collections::HashMap;
use std::error::Error;
use std::path::Path;

use csvstats::{ColumnProfile, CsvAnalysis, NumericProfile};
use plotters::coord::Shift;
use plotters::prelude::*;
use plotters::style::FontTransform;

pub fn draw_profile_dashboard(analysis: &CsvAnalysis, path: &Path) -> Result<(), Box<dyn Error>> {
    let root = BitMapBackend::new(path, (1400, 900)).into_drawing_area();
    root.fill(&WHITE)?;

    let areas = root.split_evenly((2, 2));
    draw_summary_panel(&areas[0], analysis)?;

    let labels: Vec<String> = analysis
        .profiles
        .iter()
        .map(|profile| profile.name().to_string())
        .collect();
    let null_counts: Vec<usize> = analysis.profiles.iter().map(ColumnProfile::nulls).collect();
    draw_usize_bar_chart(
        &areas[1],
        "Null Counts by Column",
        &labels,
        &null_counts,
        RGBColor(217, 91, 67),
    )?;

    draw_numeric_range_chart(&areas[2], &analysis.numeric_profiles())?;

    let categorical = analysis.categorical_profiles();
    let categorical_labels: Vec<String> = categorical
        .iter()
        .map(|profile| profile.name.clone())
        .collect();
    let unique_counts: Vec<usize> = categorical
        .iter()
        .map(|profile| profile.unique_count)
        .collect();
    draw_usize_bar_chart(
        &areas[3],
        "Categorical Cardinality",
        &categorical_labels,
        &unique_counts,
        RGBColor(52, 152, 219),
    )?;

    root.present()?;
    Ok(())
}

pub fn draw_correlation_dashboard(
    analysis: &CsvAnalysis,
    path: &Path,
) -> Result<(), Box<dyn Error>> {
    let root = BitMapBackend::new(path, (1400, 700)).into_drawing_area();
    root.fill(&WHITE)?;

    let areas = root.split_evenly((1, 2));
    draw_correlation_heatmap(&areas[0], analysis)?;
    draw_strongest_scatter(&areas[1], analysis)?;

    root.present()?;
    Ok(())
}

fn draw_summary_panel(
    area: &DrawingArea<BitMapBackend<'_>, Shift>,
    analysis: &CsvAnalysis,
) -> Result<(), Box<dyn Error>> {
    area.fill(&RGBColor(247, 249, 252))?;

    let numeric_columns = analysis.numeric_profiles();
    let categorical_columns = analysis.categorical_profiles();
    let strongest = analysis
        .strongest_correlation()
        .map(|correlation| {
            format!(
                "{} vs {}: {:.3} (n={})",
                correlation.left,
                correlation.right,
                correlation.coefficient,
                correlation.paired_count
            )
        })
        .unwrap_or_else(|| "Not enough numeric columns for correlation".to_string());

    let top_categories = categorical_columns
        .iter()
        .take(2)
        .map(|profile| {
            let label = profile
                .top_values
                .first()
                .map(|(value, count)| format!("{value} ({count})"))
                .unwrap_or_else(|| "no values".to_string());
            format!("{} top value: {}", profile.name, label)
        })
        .collect::<Vec<_>>();

    area.draw(&Text::new(
        "CSV Profile Summary",
        (24, 40),
        ("sans-serif", 30).into_font().color(&BLACK),
    ))?;

    let mut lines = vec![
        format!("Rows: {}", analysis.row_count),
        format!("Columns: {}", analysis.column_count),
        format!("Numeric columns: {}", numeric_columns.len()),
        format!("Categorical columns: {}", categorical_columns.len()),
        format!("Strongest correlation: {}", strongest),
    ];
    lines.extend(top_categories);

    for (index, line) in lines.iter().enumerate() {
        area.draw(&Text::new(
            line.as_str(),
            (24, 100 + (index as i32 * 48)),
            ("sans-serif", 22).into_font().color(&BLACK),
        ))?;
    }

    Ok(())
}

fn draw_usize_bar_chart(
    area: &DrawingArea<BitMapBackend<'_>, Shift>,
    title: &str,
    labels: &[String],
    values: &[usize],
    color: RGBColor,
) -> Result<(), Box<dyn Error>> {
    area.fill(&WHITE)?;

    if labels.is_empty() || values.is_empty() {
        return draw_empty_state(area, title, "No columns available for this chart.");
    }

    let max_value = values.iter().copied().max().unwrap_or(0);
    let upper = max_value.max(1) as u32 + 1;
    let label_lookup = labels.to_vec();

    let mut chart = ChartBuilder::on(area)
        .caption(title, ("sans-serif", 24))
        .margin(20)
        .x_label_area_size(80)
        .y_label_area_size(60)
        .build_cartesian_2d(0..labels.len() as i32, 0u32..upper)?;

    chart
        .configure_mesh()
        .disable_mesh()
        .x_labels(labels.len())
        .y_desc("Count")
        .x_label_formatter(&move |value| {
            label_lookup
                .get((*value).max(0) as usize)
                .cloned()
                .unwrap_or_default()
        })
        .x_label_style(
            ("sans-serif", 13)
                .into_font()
                .transform(FontTransform::Rotate90),
        )
        .draw()?;

    chart.draw_series(values.iter().enumerate().map(|(index, value)| {
        Rectangle::new(
            [(index as i32, 0u32), (index as i32 + 1, *value as u32)],
            color.filled(),
        )
    }))?;

    Ok(())
}

fn draw_numeric_range_chart(
    area: &DrawingArea<BitMapBackend<'_>, Shift>,
    numeric_profiles: &[&NumericProfile],
) -> Result<(), Box<dyn Error>> {
    area.fill(&WHITE)?;

    if numeric_profiles.is_empty() {
        return draw_empty_state(
            area,
            "Numeric Profile Ranges",
            "No numeric columns were detected.",
        );
    }

    let global_min = numeric_profiles
        .iter()
        .map(|profile| profile.min)
        .fold(f64::INFINITY, f64::min);
    let global_max = numeric_profiles
        .iter()
        .map(|profile| profile.max)
        .fold(f64::NEG_INFINITY, f64::max);
    let (y_min, y_max) = expand_range(global_min, global_max);
    let label_lookup: Vec<String> = numeric_profiles
        .iter()
        .map(|profile| profile.name.clone())
        .collect();

    let mut chart = ChartBuilder::on(area)
        .caption("Numeric Ranges and Means", ("sans-serif", 24))
        .margin(20)
        .x_label_area_size(80)
        .y_label_area_size(60)
        .build_cartesian_2d(0..numeric_profiles.len() as i32, y_min..y_max)?;

    chart
        .configure_mesh()
        .y_desc("Value")
        .x_labels(numeric_profiles.len())
        .disable_mesh()
        .x_label_formatter(&move |value| {
            label_lookup
                .get((*value).max(0) as usize)
                .cloned()
                .unwrap_or_default()
        })
        .x_label_style(
            ("sans-serif", 13)
                .into_font()
                .transform(FontTransform::Rotate90),
        )
        .draw()?;

    chart.draw_series(numeric_profiles.iter().enumerate().map(|(index, profile)| {
        PathElement::new(
            vec![(index as i32, profile.min), (index as i32, profile.max)],
            ShapeStyle::from(&RGBColor(44, 62, 80)).stroke_width(7),
        )
    }))?;

    chart.draw_series(numeric_profiles.iter().enumerate().map(|(index, profile)| {
        Circle::new(
            (index as i32, profile.mean),
            6,
            RGBColor(231, 76, 60).filled(),
        )
    }))?;

    Ok(())
}

fn draw_correlation_heatmap(
    area: &DrawingArea<BitMapBackend<'_>, Shift>,
    analysis: &CsvAnalysis,
) -> Result<(), Box<dyn Error>> {
    area.fill(&WHITE)?;

    let numeric_profiles = analysis.numeric_profiles();
    if numeric_profiles.len() < 2 {
        return draw_empty_state(
            area,
            "Correlation Heatmap",
            "At least two numeric columns are required.",
        );
    }

    let labels: Vec<String> = numeric_profiles
        .iter()
        .map(|profile| profile.name.clone())
        .collect();
    let mut lookup = HashMap::new();

    for correlation in &analysis.correlations {
        lookup.insert(
            (correlation.left.clone(), correlation.right.clone()),
            correlation.coefficient,
        );
        lookup.insert(
            (correlation.right.clone(), correlation.left.clone()),
            correlation.coefficient,
        );
    }

    let x_labels = labels.clone();
    let y_labels = labels.clone();
    let mut chart = ChartBuilder::on(area)
        .caption("Pearson Correlation Heatmap", ("sans-serif", 24))
        .margin(20)
        .x_label_area_size(100)
        .y_label_area_size(120)
        .build_cartesian_2d(0..labels.len() as i32, 0..labels.len() as i32)?;

    chart
        .configure_mesh()
        .disable_mesh()
        .x_labels(labels.len())
        .y_labels(labels.len())
        .x_label_formatter(&move |value| {
            x_labels
                .get((*value).max(0) as usize)
                .cloned()
                .unwrap_or_default()
        })
        .y_label_formatter(&move |value| {
            y_labels
                .get((*value).max(0) as usize)
                .cloned()
                .unwrap_or_default()
        })
        .x_label_style(
            ("sans-serif", 13)
                .into_font()
                .transform(FontTransform::Rotate90),
        )
        .draw()?;

    chart.draw_series(labels.iter().enumerate().flat_map(|(x_index, left)| {
        let lookup = &lookup;
        labels.iter().enumerate().map(move |(y_index, right)| {
            let coefficient = if x_index == y_index {
                1.0
            } else {
                *lookup.get(&(left.clone(), right.clone())).unwrap_or(&0.0)
            };
            let color = correlation_color(coefficient);
            Rectangle::new(
                [
                    (x_index as i32, y_index as i32),
                    (x_index as i32 + 1, y_index as i32 + 1),
                ],
                color.filled(),
            )
        })
    }))?;

    Ok(())
}

fn draw_strongest_scatter(
    area: &DrawingArea<BitMapBackend<'_>, Shift>,
    analysis: &CsvAnalysis,
) -> Result<(), Box<dyn Error>> {
    area.fill(&WHITE)?;

    let Some(correlation) = analysis.strongest_correlation() else {
        return draw_empty_state(
            area,
            "Strongest Scatter Plot",
            "No correlated numeric pairs were available.",
        );
    };

    let numeric_profiles = analysis.numeric_profiles();
    let left = numeric_profiles
        .iter()
        .find(|profile| profile.name == correlation.left)
        .copied();
    let right = numeric_profiles
        .iter()
        .find(|profile| profile.name == correlation.right)
        .copied();

    let (Some(left), Some(right)) = (left, right) else {
        return draw_empty_state(
            area,
            "Strongest Scatter Plot",
            "Profile data could not be matched.",
        );
    };

    let points: Vec<(f64, f64)> = left
        .values
        .iter()
        .zip(&right.values)
        .filter_map(
            |(left_value, right_value)| match (left_value, right_value) {
                (Some(x), Some(y)) => Some((*x, *y)),
                _ => None,
            },
        )
        .collect();

    if points.is_empty() {
        return draw_empty_state(
            area,
            "Strongest Scatter Plot",
            "No paired numeric data points were found.",
        );
    }

    let x_min = points.iter().map(|(x, _)| *x).fold(f64::INFINITY, f64::min);
    let x_max = points
        .iter()
        .map(|(x, _)| *x)
        .fold(f64::NEG_INFINITY, f64::max);
    let y_min = points.iter().map(|(_, y)| *y).fold(f64::INFINITY, f64::min);
    let y_max = points
        .iter()
        .map(|(_, y)| *y)
        .fold(f64::NEG_INFINITY, f64::max);
    let (x_min, x_max) = expand_range(x_min, x_max);
    let (y_min, y_max) = expand_range(y_min, y_max);

    let mut chart = ChartBuilder::on(area)
        .caption(
            format!(
                "Strongest Pair: {} vs {} (r = {:.3})",
                correlation.left, correlation.right, correlation.coefficient
            ),
            ("sans-serif", 24),
        )
        .margin(20)
        .x_label_area_size(60)
        .y_label_area_size(70)
        .build_cartesian_2d(x_min..x_max, y_min..y_max)?;

    chart
        .configure_mesh()
        .x_desc(correlation.left.as_str())
        .y_desc(correlation.right.as_str())
        .draw()?;

    chart.draw_series(
        points
            .iter()
            .map(|point| Circle::new(*point, 3, RGBColor(41, 128, 185).mix(0.7).filled())),
    )?;

    Ok(())
}

fn draw_empty_state(
    area: &DrawingArea<BitMapBackend<'_>, Shift>,
    title: &str,
    message: &str,
) -> Result<(), Box<dyn Error>> {
    area.fill(&WHITE)?;
    area.draw(&Text::new(
        title,
        (24, 40),
        ("sans-serif", 28).into_font().color(&BLACK),
    ))?;
    area.draw(&Text::new(
        message,
        (24, 100),
        ("sans-serif", 22).into_font().color(&BLACK),
    ))?;
    Ok(())
}

fn expand_range(min: f64, max: f64) -> (f64, f64) {
    if !min.is_finite() || !max.is_finite() {
        return (0.0, 1.0);
    }

    if (max - min).abs() < f64::EPSILON {
        let padding = if min == 0.0 { 1.0 } else { min.abs() * 0.1 };
        (min - padding, max + padding)
    } else {
        let padding = (max - min) * 0.1;
        (min - padding, max + padding)
    }
}

fn correlation_color(coefficient: f64) -> RGBColor {
    let normalized = ((coefficient + 1.0) / 2.0).clamp(0.0, 1.0);
    let red = (normalized * 255.0) as u8;
    let blue = ((1.0 - normalized) * 255.0) as u8;
    let green = (120.0 * (1.0 - (coefficient.abs() * 0.6))) as u8;
    RGBColor(red, green, blue)
}
