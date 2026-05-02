use csvprof::{StatsAnalyzer}; // Reusing your Part 1/2 logic
use plotters::prelude::*;
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // --- 1. MOCK DATA / DATA LOADING ---
    // In a real scenario, you'd call your CSV loading functions here.
    // Let's assume we've aggregated data by Neighborhood.
    let mut correlation_data = vec![
        ("Canton", 150, 450000),
        ("Fells Point", 200, 380000),
        ("Sandtown", 600, 80000),
        ("Bolton Hill", 120, 520000),
        ("Brooklyn", 450, 110000),
    ];

    // --- 2. SETUP DRAWING AREA ---
    let root = BitMapBackend::new("reports/analysis_viz.png", (1024, 768)).into_drawing_area();
    root.fill(&WHITE)?;
    let (upper, lower) = root.split_vertically(384);

    // --- 3. PLOT 1: PROFILE STATISTICS (Bar Chart) ---
    // Showing volume of 311 requests per neighborhood
    let mut chart1 = ChartBuilder::on(&upper)
        .caption("Profile Stats: 311 Requests by Neighborhood", ("sans-serif", 30).into_font())
        .margin(20)
        .x_label_area_size(40)
        .y_label_area_size(40)
        .build_cartesian_2d(
            correlation_data.iter().map(|(n, _, _)| *n).collect::<Vec<_>>().into_segmented(),
            0..700,
        )?;

    chart1.configure_mesh().draw()?;

    chart1.draw_series(
        correlation_data.iter().map(|(name, count, _)| {
            Rectangle::new(
                [(SegmentValue::Exact(name), 0), (SegmentValue::Exact(name), *count as i32)],
                RED.filled(),
            )
        }),
    )?;

    // --- 4. PLOT 2: CORRELATION (Scatter Plot) ---
    // X-axis: Request Count, Y-axis: Property Value
    let mut chart2 = ChartBuilder::on(&lower)
        .caption("Correlation: Request Volume vs. Property Value", ("sans-serif", 30).into_font())
        .margin(20)
        .x_label_area_size(40)
        .y_label_area_size(40)
        .build_cartesian_2d(0..700, 0..600000)?;

    chart2.configure_mesh()
        .x_desc("Number of 311 Requests")
        .y_desc("Avg Property Assessment ($)")
        .draw()?;

    chart2.draw_series(
        correlation_data.iter().map(|(_, count, val)| {
            Circle::new((*count as i32, *val as i32), 5, BLUE.filled())
        }),
    )?;

    root.present()?;
    println!("Visualization saved to reports/analysis_viz.png");
    Ok(())
}