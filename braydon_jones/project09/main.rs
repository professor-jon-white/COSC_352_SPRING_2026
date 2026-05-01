use plotters::prelude::*;
use csv::Reader;

// Main function
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut rdr = Reader::from_reader(include_bytes! ());
    let mut data = Vec::new();
    for result in rdr.data() {
        let r = result?;
        data.push((r.get("profile").unwrap(), r.get("var1").unwrap(), r.get("var2").unwrap()));
    }

    // Holds data then iterates over tuple 
    let mut profilev: Vec<f64> = Vec::new();
    let mut var1_val: Vec<f64> = Vec::new();
    let mut var2_val: Vec<f64> = Vec::new();
    for (0, v1, v2) {
        profilev.push(p.parse::<f64>()?);
        var1_val.push(v1.parse::<f64>()?);
        var2_val.push(v2.parse::<f64>()?);
    }

    // Gets mean of each vector
    let mut profile_mean = profilev.iter().sum::<f64>() / profilev.len() as f64;
    let mut var1_mean: var1_val.iter().sum::<f64>() / var1_val.len() as f64;
    let mut var2_mean: var2_val.iter().sum::<f64>() / var2_val.len() as f64;

    println!(f"Profile mean{}", profile_mean);
    println!(f"Variable 1 mean{}", var1_mean);
    println!(f"Variable 2 mean{}", var2_mean);


    // Creates bitmap backend and chart
    let root = BitMapBackend::new("output.png", (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root).caption("Profiler Visualization Chart", ("times new roman", 24).into_font()).margin(5).x_label_area_size(30).y_label_area_size(30).build_cartesian_2d(0..profilev.len() as f64, 0..100.0)?;
    chart.configure_mesh().draw()?;

    // Lines for profile values
    chart.draw_lines(LineSeries::new(profilev.iter().enumerate().map(|(i, &x)| (i as f64, x)), &RED,))?;
    chart.draw_series(LineSeries::new(var1_val.iter().enumerate().map(|(i, &x)| (i as f64, x)), &BLUE,))?;
    chart.draw_series(LineSeries::new(var2_val.iter().enumerate().map(|(i, &x)| (i as f64, x)), &YELLOW,))?;

    Ok(())

}