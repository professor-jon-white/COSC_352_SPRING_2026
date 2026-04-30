mod render;

use std::error::Error;
use std::fs;
use std::path::PathBuf;

use clap::Parser;
use csvstats::analyze_csv;

#[derive(Parser, Debug)]
#[command(name = "csvviz")]
#[command(about = "Profile CSV data and render Plotters dashboards")]
struct Args {
    /// Path to the CSV file you want to visualize
    file: String,
    /// Directory where PNG dashboards will be written
    #[arg(short, long, default_value = "output")]
    out_dir: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let analysis = analyze_csv(&args.file)?;

    fs::create_dir_all(&args.out_dir)?;

    let profile_path = args.out_dir.join("profile_dashboard.png");
    let correlation_path = args.out_dir.join("correlation_dashboard.png");

    render::draw_profile_dashboard(&analysis, &profile_path)?;
    render::draw_correlation_dashboard(&analysis, &correlation_path)?;

    println!("Wrote {}", profile_path.display());
    println!("Wrote {}", correlation_path.display());

    Ok(())
}
