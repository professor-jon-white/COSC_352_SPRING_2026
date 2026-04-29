use anyhow::{Context, Result};
use csv::StringRecord;
use csvprof::profiler::{profile_csv, ProfileConfig};
use std::collections::HashMap;

const REQUESTS_FILE: &str = "data/311_requests_2025.csv";
const VACANTS_FILE: &str = "data/vacant_building_notices.csv";

#[derive(Debug)]
struct DistrictStats {
    district: String,
    request_count: u64,
    vacant_count: u64,
}

fn main() -> Result<()> {
    let cfg = ProfileConfig {
        delimiter: b',',
        has_headers: true,
        percentiles: false,
        histogram: false,
        max_categories: 20,
        categorical_ratio: 0.05,
    };

    let requests_profile = profile_csv(REQUESTS_FILE, &cfg)?;
    let vacants_profile = profile_csv(VACANTS_FILE, &cfg)?;

    println!("PROJECT 8 ANALYSIS");
    println!("==================");
    println!("Rows (311): {}", requests_profile.total_rows);
    println!("Rows (Vacant): {}", vacants_profile.total_rows);
    println!();

    let requests_by_district = count_by_council_district(REQUESTS_FILE)?;
    let vacants_by_district = count_by_council_district(VACANTS_FILE)?;

    let mut joined: Vec<DistrictStats> = Vec::new();

    for (district, vacant_count) in vacants_by_district {
        let request_count = requests_by_district.get(&district).copied().unwrap_or(0);

        joined.push(DistrictStats {
            district,
            request_count,
            vacant_count,
        });
    }

    joined.sort_by(|a, b| b.vacant_count.cmp(&a.vacant_count));

    println!("Research Question:");
    println!("Do Baltimore council districts with more vacant building notices also have more 311 service requests?");
    println!();

    println!("{:<18} {:>15} {:>15}", "Council District", "Vacant", "311");
    println!("{}", "-".repeat(52));

    for row in joined.iter().take(15) {
        println!(
            "{:<18} {:>15} {:>15}",
            row.district, row.vacant_count, row.request_count
        );
    }

    let correlation = pearson(&joined);

    println!();
    println!("Correlation: {:.4}", correlation);

    if let Some(top) = joined.first() {
        println!(
            "Top district by vacant notices: District {} with {} vacant notices and {} 311 requests.",
            top.district, top.vacant_count, top.request_count
        );
    }

    println!();

    if correlation >= 0.50 {
        println!("Answer: The data shows a strong positive relationship.");
    } else if correlation >= 0.20 {
        println!("Answer: The data shows a weak-to-moderate positive relationship.");
    } else if correlation <= -0.20 {
        println!("Answer: The data shows a negative relationship.");
    } else {
        println!("Answer: The data shows little or no clear linear relationship.");
    }

    Ok(())
}

fn count_by_council_district(path: &str) -> Result<HashMap<String, u64>> {
    let mut rdr = csv::ReaderBuilder::new()
        .flexible(true)
        .from_path(path)?;

    let headers = rdr.headers()?.clone();

    let idx = find_council_district(&headers)
        .with_context(|| format!("Council District column not found in {}", path))?;

    let mut counts: HashMap<String, u64> = HashMap::new();

    for result in rdr.records() {
        let record = result?;

        if let Some(value) = record.get(idx) {
            let district = clean_district(value);

            if !district.is_empty() {
                *counts.entry(district).or_insert(0) += 1;
            }
        }
    }

    Ok(counts)
}

fn find_council_district(headers: &StringRecord) -> Option<usize> {
    headers.iter().position(|h| {
        let cleaned = h
            .to_lowercase()
            .replace('_', "")
            .replace(' ', "")
            .replace('-', "");

        cleaned == "councildistrict" || cleaned.contains("councildistrict")
    })
}

fn clean_district(value: &str) -> String {
    value
        .chars()
        .filter(|c| c.is_ascii_digit())
        .collect::<String>()
}

fn pearson(data: &[DistrictStats]) -> f64 {
    if data.len() < 2 {
        return 0.0;
    }

    let n = data.len() as f64;

    let mean_x = data.iter().map(|r| r.vacant_count as f64).sum::<f64>() / n;
    let mean_y = data.iter().map(|r| r.request_count as f64).sum::<f64>() / n;

    let mut num = 0.0;
    let mut dx = 0.0;
    let mut dy = 0.0;

    for r in data {
        let x = r.vacant_count as f64 - mean_x;
        let y = r.request_count as f64 - mean_y;

        num += x * y;
        dx += x * x;
        dy += y * y;
    }

    if dx == 0.0 || dy == 0.0 {
        0.0
    } else {
        num / (dx.sqrt() * dy.sqrt())
    }
}