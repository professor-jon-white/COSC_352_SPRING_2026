use csv::ReaderBuilder;
use std::collections::HashMap;
use std::io::BufReader;

#[derive(Debug, Clone)]
pub struct ZipLiquorData {
    pub zip: String,
    pub total_licenses: usize,
    pub active_licenses: usize,
    pub tavern_count: usize,
    pub restaurant_count: usize,
    pub package_goods_count: usize,
    pub avg_license_fee: f64,
}

#[derive(Debug, Clone)]
pub struct SalaryData {
    pub total_employees: usize,
    pub police_count: usize,
    pub fire_count: usize,
    pub citywide_avg_salary: f64,
    pub police_avg_salary: f64,
    pub fire_avg_salary: f64,
    pub above_avg_police: usize,
    pub above_avg_fire: usize,
}

pub type Row = HashMap<String, String>;

pub fn load_liquor_licenses(path: &str) -> Result<Vec<Row>, Box<dyn std::error::Error>> {
    let file = std::fs::File::open(path)?;
    let buf = BufReader::new(file);
    let mut reader = ReaderBuilder::new().from_reader(buf);

    let mut rows = Vec::new();
    for result in reader.deserialize() {
        let record: Row = result?;
        rows.push(record);
    }

    Ok(rows)
}

pub fn load_salaries(path: &str) -> Result<Vec<Row>, Box<dyn std::error::Error>> {
    let file = std::fs::File::open(path)?;
    let buf = BufReader::new(file);
    let mut reader = ReaderBuilder::new().from_reader(buf);

    let mut rows = Vec::new();
    for result in reader.deserialize() {
        let record: Row = result?;
        rows.push(record);
    }

    Ok(rows)
}

pub fn compute_zip_stats(liquor_rows: &[Row]) -> Vec<ZipLiquorData> {
    let mut map: HashMap<String, ZipLiquorData> = HashMap::new();

    for row in liquor_rows {
        let Some(zip) = row.get("AddrZip") else { continue; };
        let zip = zip.trim().to_string();
        
        if zip.is_empty() || !zip.chars().all(|c| c.is_ascii_digit()) {
            continue;
        }

        let status = row.get("LicenseStatus").map(|s| s.trim()).unwrap_or("");
        let desc = row.get("EstablishmentDesc").map(|s| s.trim()).unwrap_or("");
        let fee = row
            .get("LicenseFee")
            .and_then(|s| s.trim().parse::<f64>().ok())
            .unwrap_or(0.0);

        let entry = map.entry(zip.clone()).or_insert(ZipLiquorData {
            zip: zip.clone(),
            total_licenses: 0,
            active_licenses: 0,
            tavern_count: 0,
            restaurant_count: 0,
            package_goods_count: 0,
            avg_license_fee: 0.0,
        });

        entry.total_licenses += 1;
        if status == "Renewed" {
            entry.active_licenses += 1;
        }
        if desc == "Tavern" || desc == "Tavern License" {
            entry.tavern_count += 1;
        }
        if desc == "Restaurant" || desc == "Restaurant License" {
            entry.restaurant_count += 1;
        }
        if desc == "Package goods only" || desc == "Package Goods Only" {
            entry.package_goods_count += 1;
        }
        entry.avg_license_fee += fee;
    }

    for stat in map.values_mut() {
        if stat.total_licenses > 0 {
            stat.avg_license_fee /= stat.total_licenses as f64;
        }
    }

    let mut result: Vec<ZipLiquorData> = map.into_values().collect();
    result.sort_by_key(|a| std::cmp::Reverse(a.active_licenses));
    result
}

pub fn compute_salary_stats(salary_rows: &[Row]) -> SalaryData {
    let fy2024: Vec<&Row> = salary_rows
        .iter()
        .filter(|row| row.get("FiscalYear").map(|s| s.trim()) == Some("FY2024"))
        .collect();

    let all_salaries: Vec<f64> = fy2024
        .iter()
        .filter_map(|row| row.get("AnnualSalary"))
        .filter_map(|value| value.trim().parse::<f64>().ok())
        .collect();

    let citywide_avg = if all_salaries.is_empty() {
        0.0
    } else {
        all_salaries.iter().sum::<f64>() / all_salaries.len() as f64
    };

    let police: Vec<f64> = fy2024
        .iter()
        .filter(|row| row.get("AgencyName").map(|s| s.trim()) == Some("Police Department"))
        .filter_map(|row| row.get("AnnualSalary"))
        .filter_map(|value| value.trim().parse::<f64>().ok())
        .collect();

    let fire: Vec<f64> = fy2024
        .iter()
        .filter(|row| row.get("AgencyName").map(|s| s.trim()) == Some("Fire Department"))
        .filter_map(|row| row.get("AnnualSalary"))
        .filter_map(|value| value.trim().parse::<f64>().ok())
        .collect();

    let police_avg = if police.is_empty() {
        0.0
    } else {
        police.iter().sum::<f64>() / police.len() as f64
    };

    let fire_avg = if fire.is_empty() {
        0.0
    } else {
        fire.iter().sum::<f64>() / fire.len() as f64
    };

    let above_avg_police = police.iter().filter(|&&salary| salary > citywide_avg).count();
    let above_avg_fire = fire.iter().filter(|&&salary| salary > citywide_avg).count();

    SalaryData {
        total_employees: fy2024.len(),
        police_count: police.len(),
        fire_count: fire.len(),
        citywide_avg_salary: citywide_avg,
        police_avg_salary: police_avg,
        fire_avg_salary: fire_avg,
        above_avg_police,
        above_avg_fire,
    }
}
