use std::collections::HashMap;
use std::error::Error;
use csv::Reader;

pub fn run_analysis() -> Result<(), Box<dyn Error>> {
    let mut crime_counts: HashMap<String, usize> = HashMap::new();
    let mut call_counts: HashMap<String, usize> = HashMap::new();

    // DATASET 1 = Crime Data
    let mut rdr1 = Reader::from_path("data/dataset1.csv")?;
    let headers1 = rdr1.headers()?.clone();
    let neighborhood_idx1 = headers1
        .iter()
        .position(|h| h.trim() == "Neighborhood")
        .ok_or("Neighborhood column not found in dataset1")?;

    for result in rdr1.records() {
        let record = result?;
        let neighborhood = record
            .get(neighborhood_idx1)
            .unwrap_or("")
            .trim()
            .to_uppercase();
        if !neighborhood.is_empty() && neighborhood != "N/A" && neighborhood != "UNKNOWN" {
            *crime_counts.entry(neighborhood.to_string()).or_insert(0) += 1;
        }
    }

    // DATASET 2 = 911 Calls
    let mut rdr2 = Reader::from_path("data/dataset2.csv")?;
    let headers2 = rdr2.headers()?.clone();
    let neighborhood_idx2 = headers2
        .iter()
        .position(|h| h.trim() == "Neighborhood")
        .ok_or("Neighborhood column not found in dataset2")?;

    for result in rdr2.records() {
        let record = result?;
        let neighborhood = record
            .get(neighborhood_idx2)
            .unwrap_or("")
            .trim()
            .to_uppercase();
        if !neighborhood.is_empty() && neighborhood != "N/A" && neighborhood != "UNKNOWN" {
            *call_counts.entry(neighborhood.to_string()).or_insert(0) += 1;
        }
    }

    println!("Baltimore Neighborhood Crime vs 911 Calls Analysis");
    println!("=================================================");
    println!("Neighborhood | Crimes | 911 Calls");

    let mut combined: Vec<(String, usize, usize)> = Vec::new();

    for (neighborhood, crime_count) in &crime_counts {
        if let Some(call_count) = call_counts.get(neighborhood) {
            combined.push((
                neighborhood.clone(),
                *crime_count,
                *call_count,
            ));
        }
    }

    combined.sort_by(|a, b| b.1.cmp(&a.1));

    for (neighborhood, crimes, calls) in combined.iter().take(10) {
        println!("{} | {} | {}", neighborhood, crimes, calls);
    }

    Ok(())
}