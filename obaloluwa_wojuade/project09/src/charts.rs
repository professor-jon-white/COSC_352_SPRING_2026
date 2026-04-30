use crate::data::{SalaryData, ZipLiquorData};
use plotters::prelude::*;
use std::error::Error;

pub fn chart_1_top_zip_codes(zips: &[ZipLiquorData]) -> Result<(), Box<dyn Error>> {
    let root = BitMapBackend::new("output/01_top_zip_codes.png", (1400, 700))
        .into_drawing_area();
    root.fill(&WHITE)?;

    let top_15 = &zips[..zips.len().min(15)];
    let max_active = top_15.iter().map(|z| z.active_licenses).max().unwrap_or(1) as i32;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            "Top 15 Baltimore Zip Codes by Active Liquor Licenses",
            ("sans-serif", 24).into_font(),
        )
        .margin(15)
        .x_label_area_size(60)
        .y_label_area_size(70)
        .build_cartesian_2d(
            0usize..15usize,
            0i32..((max_active as f64 * 1.1) as i32),
        )?;

    chart
        .configure_mesh()
        .x_desc("Zip Code")
        .y_desc("Active Licenses")
        .x_label_formatter(&|x| {
            if *x < top_15.len() {
                top_15[*x].zip.clone()
            } else {
                "".to_string()
            }
        })
        .draw()?;

    chart.draw_series((0..top_15.len()).map(|idx| {
        let zip = &top_15[idx];
        Rectangle::new(
            [(idx, 0), (idx + 1, zip.active_licenses as i32)],
            BLUE.filled(),
        )
    }))?;

    root.present()?;
    Ok(())
}

pub fn chart_2_license_types(zips: &[ZipLiquorData]) -> Result<(), Box<dyn Error>> {
    let root = BitMapBackend::new("output/02_license_types_top_5.png", (1400, 700))
        .into_drawing_area();
    root.fill(&WHITE)?;

    let top_5 = &zips[..zips.len().min(5)];
    let max_licenses = top_5.iter().map(|z| z.total_licenses).max().unwrap_or(1) as i32;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            "License Type Distribution - Top 5 Zip Codes",
            ("sans-serif", 24).into_font(),
        )
        .margin(15)
        .x_label_area_size(60)
        .y_label_area_size(70)
        .build_cartesian_2d(
            0usize..5usize,
            0i32..((max_licenses as f64 * 1.1) as i32),
        )?;

    chart
        .configure_mesh()
        .x_desc("Zip Code")
        .y_desc("Number of Licenses")
        .x_label_formatter(&|x| {
            if *x < top_5.len() {
                top_5[*x].zip.clone()
            } else {
                "".to_string()
            }
        })
        .draw()?;

    // Draw stacked bars: taverns + restaurants + package goods
    for (idx, z) in top_5.iter().enumerate() {
        let tavern_height = z.tavern_count as i32;
        let restaurant_height = z.restaurant_count as i32;
        let package_height = z.package_goods_count as i32;

        // Draw tavern bar (red)
        let _ = chart.draw_series(std::iter::once(Rectangle::new(
            [(idx, 0), (idx + 1, tavern_height)],
            RED.filled(),
        )));

        // Draw restaurant bar (green) on top
        let _ = chart.draw_series(std::iter::once(Rectangle::new(
            [(idx, tavern_height), (idx + 1, tavern_height + restaurant_height)],
            GREEN.filled(),
        )));

        // Draw package goods bar (blue) on top
        let _ = chart.draw_series(std::iter::once(Rectangle::new(
            [
                (idx, tavern_height + restaurant_height),
                (idx + 1, tavern_height + restaurant_height + package_height),
            ],
            BLUE.filled(),
        )));
    }

    root.present()?;
    Ok(())
}

pub fn chart_3_license_fees(zips: &[ZipLiquorData]) -> Result<(), Box<dyn Error>> {
    let root = BitMapBackend::new("output/03_license_fees.png", (1400, 700))
        .into_drawing_area();
    root.fill(&WHITE)?;

    let top_10 = &zips[..zips.len().min(10)];
    let max_fee = top_10
        .iter()
        .map(|z| (z.avg_license_fee * 10.0) as i32)
        .max()
        .unwrap_or(1);

    let mut chart = ChartBuilder::on(&root)
        .caption(
            "Average License Fees by Zip Code (Top 10)",
            ("sans-serif", 24).into_font(),
        )
        .margin(15)
        .x_label_area_size(60)
        .y_label_area_size(70)
        .build_cartesian_2d(
            0usize..10usize,
            0i32..((max_fee as f64 * 1.1) as i32),
        )?;

    chart
        .configure_mesh()
        .x_desc("Zip Code")
        .y_desc("Average License Fee ($)")
        .x_label_formatter(&|x| {
            if *x < top_10.len() {
                top_10[*x].zip.clone()
            } else {
                "".to_string()
            }
        })
        .y_label_formatter(&|y| format!("${}", y * 100))
        .draw()?;

    chart.draw_series((0..top_10.len()).map(|idx| {
        let zip = &top_10[idx];
        Rectangle::new(
            [(idx, 0), (idx + 1, (zip.avg_license_fee * 10.0) as i32)],
            GREEN.filled(),
        )
    }))?;

    root.present()?;
    Ok(())
}

pub fn chart_4_salary_comparison(salary_data: &SalaryData) -> Result<(), Box<dyn Error>> {
    let root = BitMapBackend::new("output/04_salary_comparison.png", (1200, 700))
        .into_drawing_area();
    root.fill(&WHITE)?;

    let max_salary = [
        salary_data.citywide_avg_salary,
        salary_data.police_avg_salary,
        salary_data.fire_avg_salary,
    ]
    .iter()
    .copied()
    .fold(f64::NEG_INFINITY, f64::max) as i32;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            "Public Safety: Average Salary Comparison (FY2024)",
            ("sans-serif", 24).into_font(),
        )
        .margin(15)
        .x_label_area_size(60)
        .y_label_area_size(70)
        .build_cartesian_2d(0usize..3usize, 0i32..((max_salary as f64 * 1.15) as i32))?;

    chart
        .configure_mesh()
        .x_desc("Department")
        .y_desc("Average Annual Salary ($)")
        .x_label_formatter(&|x| match x {
            0 => "Citywide Avg".to_string(),
            1 => "Police Dept".to_string(),
            2 => "Fire Dept".to_string(),
            _ => "".to_string(),
        })
        .y_label_formatter(&|y| format!("${}", y * 1000))
        .draw()?;

    let salaries = [
        (0, salary_data.citywide_avg_salary as i32),
        (1, salary_data.police_avg_salary as i32),
        (2, salary_data.fire_avg_salary as i32),
    ];

    chart.draw_series(salaries.iter().map(|(x, height)| {
        Rectangle::new([(*x, 0), (*x + 1, *height)], CYAN.filled())
    }))?;

    root.present()?;
    Ok(())
}

pub fn chart_5_above_avg_earners(salary_data: &SalaryData) -> Result<(), Box<dyn Error>> {
    let root = BitMapBackend::new("output/05_above_avg_earners.png", (1200, 700))
        .into_drawing_area();
    root.fill(&WHITE)?;

    let max_count = salary_data
        .above_avg_police
        .max(salary_data.above_avg_fire) as i32;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            "Employees Earning Above Citywide Average Salary",
            ("sans-serif", 24).into_font(),
        )
        .margin(15)
        .x_label_area_size(60)
        .y_label_area_size(70)
        .build_cartesian_2d(0usize..3usize, 0i32..((max_count as f64 * 1.15) as i32))?;

    chart
        .configure_mesh()
        .x_desc("Department")
        .y_desc("Number of Above-Average Earners")
        .x_label_formatter(&|x| match x {
            0 => "Police".to_string(),
            1 => "Fire".to_string(),
            2 => "Total Public Safety".to_string(),
            _ => "".to_string(),
        })
        .draw()?;

    let counts = [
        (0, salary_data.above_avg_police as i32),
        (1, salary_data.above_avg_fire as i32),
        (2, (salary_data.above_avg_police + salary_data.above_avg_fire) as i32),
    ];

    chart.draw_series(counts.iter().map(|(x, height)| {
        Rectangle::new([(*x, 0), (*x + 1, *height)], MAGENTA.filled())
    }))?;

    root.present()?;
    Ok(())
}

pub fn chart_6_active_vs_total(zips: &[ZipLiquorData]) -> Result<(), Box<dyn Error>> {
    let root = BitMapBackend::new("output/06_active_vs_total_licenses.png", (1400, 700))
        .into_drawing_area();
    root.fill(&WHITE)?;

    let top_10 = &zips[..zips.len().min(10)];
    let max_licenses = top_10
        .iter()
        .map(|z| z.total_licenses.max(z.active_licenses))
        .max()
        .unwrap_or(1) as i32;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            "Total vs Active Licenses - Top 10 Zip Codes",
            ("sans-serif", 24).into_font(),
        )
        .margin(15)
        .x_label_area_size(60)
        .y_label_area_size(70)
        .build_cartesian_2d(0usize..20usize, 0i32..((max_licenses as f64 * 1.1) as i32))?;

    chart
        .configure_mesh()
        .x_desc("Zip Code")
        .y_desc("License Count")
        .x_label_formatter(&|x| {
            let idx = x / 2;
            if x % 2 == 0 && idx < top_10.len() {
                top_10[idx].zip.clone()
            } else {
                "".to_string()
            }
        })
        .draw()?;

    // Draw pairs of bars: total (blue) and active (red) side by side
    for (idx, z) in top_10.iter().enumerate() {
        let x_total = (idx * 2) as usize;
        let x_active = (idx * 2 + 1) as usize;

        // Total licenses (blue)
        let _ = chart.draw_series(std::iter::once(Rectangle::new(
            [(x_total, 0), (x_total + 1, z.total_licenses as i32)],
            BLUE.filled(),
        )));

        // Active licenses (red)
        let _ = chart.draw_series(std::iter::once(Rectangle::new(
            [(x_active, 0), (x_active + 1, z.active_licenses as i32)],
            RED.filled(),
        )));
    }

    root.present()?;
    Ok(())
}
