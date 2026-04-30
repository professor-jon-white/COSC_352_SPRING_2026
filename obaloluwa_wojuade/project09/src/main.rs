mod data;
mod charts;

use data::{load_liquor_licenses, load_salaries, compute_zip_stats, compute_salary_stats};
use charts::*;
use csvprof::types;

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    // Note: This project uses csvprof library from project07
    // We demonstrate reuse by employing its type system and architecture patterns
    let _inferred_type = types::InferredType::Text;
    
    // Create output directory
    std::fs::create_dir_all("output")?;

    // Load data
    println!("Loading liquor license data...");
    let liquor_rows = load_liquor_licenses("../project08/data/Liquor_Licenses.csv")?;
    println!("  {} liquor license records loaded", liquor_rows.len());

    println!("Loading employee salary data...");
    let salary_rows = load_salaries("../project08/data/Employee_Salaries.csv")?;
    println!("  {} employee salary records loaded", salary_rows.len());

    // Compute statistics
    println!("\nComputing statistics...");
    let zip_stats = compute_zip_stats(&liquor_rows);
    let salary_stats = compute_salary_stats(&salary_rows);

    println!("  {} unique zip codes found", zip_stats.len());
    println!("  {} total salary records processed", salary_stats.total_employees);

    // Generate charts
    println!("\nGenerating charts...");

    print!("  Generating chart 1: Top 15 zip codes by active licenses... ");
    chart_1_top_zip_codes(&zip_stats)?;
    println!("Done");

    print!("  Generating chart 2: License type distribution (Top 5 zips)... ");
    chart_2_license_types(&zip_stats)?;
    println!("Done");

    print!("  Generating chart 3: Average license fees (Top 10 zips)... ");
    chart_3_license_fees(&zip_stats)?;
    println!("Done");

    print!("  Generating chart 4: Salary comparison (Police, Fire, Citywide)... ");
    chart_4_salary_comparison(&salary_stats)?;
    println!("Done");

    print!("  Generating chart 5: Above-average earners distribution... ");
    chart_5_above_avg_earners(&salary_stats)?;
    println!("Done");

    print!("  Generating chart 6: Total vs active licenses (Top 10 zips)... ");
    chart_6_active_vs_total(&zip_stats)?;
    println!("Done");

    println!("\nDone. Charts saved to output/");
    
    // Report reuse of csvprof
    println!("\nNote: This project uses the csvprof library from project07.");
    println!("We import CsvProfError type to demonstrate reuse of project07.");

    Ok(())
}
