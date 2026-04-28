use std::collections::HashMap;

use chrono::NaiveDate;

use crate::value::{InferredType, PrimitiveKind, ValueInspection};

pub trait Metric {
    fn observe(&mut self, inspection: &ValueInspection<'_>);
}

#[derive(Debug, Clone, Default)]
pub struct TypeCounter {
    integer_count: u64,
    float_count: u64,
    boolean_count: u64,
    date_count: u64,
    text_count: u64,
}

impl Metric for TypeCounter {
    fn observe(&mut self, inspection: &ValueInspection<'_>) {
        match inspection.primary_kind() {
            PrimitiveKind::Integer => self.integer_count += 1,
            PrimitiveKind::Float => self.float_count += 1,
            PrimitiveKind::Boolean => self.boolean_count += 1,
            PrimitiveKind::Date => self.date_count += 1,
            PrimitiveKind::Text => self.text_count += 1,
        }
    }
}

impl TypeCounter {
    pub fn infer_type(
        &self,
        non_null_count: u64,
        unique_count: u64,
        threshold: usize,
    ) -> InferredType {
        if non_null_count == 0 {
            return InferredType::Text;
        }

        let numeric_count = self.integer_count + self.float_count;
        let numeric_ratio = numeric_count as f64 / non_null_count as f64;
        let boolean_ratio = self.boolean_count as f64 / non_null_count as f64;
        let date_ratio = self.date_count as f64 / non_null_count as f64;

        if self.boolean_count == non_null_count {
            return InferredType::Boolean;
        }

        if self.date_count == non_null_count {
            return InferredType::Date;
        }

        if self.text_count == 0
            && self.boolean_count == 0
            && self.date_count == 0
            && numeric_count > 0
        {
            return if self.float_count > 0 {
                InferredType::Float
            } else {
                InferredType::Integer
            };
        }

        if numeric_ratio >= 0.90 && self.boolean_count == 0 && self.date_count == 0 {
            return if self.float_count > 0 {
                InferredType::Float
            } else {
                InferredType::Integer
            };
        }

        if boolean_ratio >= 0.90 && numeric_count == 0 && self.date_count == 0 {
            return InferredType::Boolean;
        }

        if date_ratio >= 0.90 && numeric_count == 0 && self.boolean_count == 0 {
            return InferredType::Date;
        }

        if is_low_cardinality(unique_count, non_null_count, threshold) {
            InferredType::Categorical
        } else {
            InferredType::Text
        }
    }

    pub fn analyzed_count(&self, inferred_type: InferredType) -> u64 {
        match inferred_type {
            InferredType::Integer => self.integer_count,
            InferredType::Float => self.integer_count + self.float_count,
            InferredType::Boolean => self.boolean_count,
            InferredType::Date => self.date_count,
            InferredType::Categorical | InferredType::Text => self.total_count(),
        }
    }

    pub fn mixed_type_warning(
        &self,
        inferred_type: InferredType,
        non_null_count: u64,
    ) -> Option<String> {
        let observed = self.observed_kinds();
        if observed.len() <= 1 {
            return None;
        }

        let labels = join_kinds(&observed);

        if observed == vec![PrimitiveKind::Integer, PrimitiveKind::Float] {
            return Some(
                "integer and float values were both observed, so the column was promoted to float"
                    .to_string(),
            );
        }

        let analyzed = self.analyzed_count(inferred_type);
        let coverage = analyzed as f64 / non_null_count as f64 * 100.0;

        Some(format!(
            "mixed values detected ({labels}); inferred {inferred_type} because it covers {coverage:.1}% of non-null cells"
        ))
    }

    pub fn total_count(&self) -> u64 {
        self.integer_count
            + self.float_count
            + self.boolean_count
            + self.date_count
            + self.text_count
    }

    fn observed_kinds(&self) -> Vec<PrimitiveKind> {
        let mut kinds = Vec::new();
        if self.integer_count > 0 {
            kinds.push(PrimitiveKind::Integer);
        }
        if self.float_count > 0 {
            kinds.push(PrimitiveKind::Float);
        }
        if self.boolean_count > 0 {
            kinds.push(PrimitiveKind::Boolean);
        }
        if self.date_count > 0 {
            kinds.push(PrimitiveKind::Date);
        }
        if self.text_count > 0 {
            kinds.push(PrimitiveKind::Text);
        }
        kinds
    }
}

#[derive(Debug, Clone, Default)]
pub struct NumericStats {
    count: u64,
    mean: f64,
    m2: f64,
    min: Option<f64>,
    max: Option<f64>,
    values: Vec<f64>,
}

impl Metric for NumericStats {
    fn observe(&mut self, inspection: &ValueInspection<'_>) {
        let Some(value) = inspection.numeric_value() else {
            return;
        };

        self.count += 1;
        self.values.push(value);
        self.min = Some(match self.min {
            Some(current) => current.min(value),
            None => value,
        });
        self.max = Some(match self.max {
            Some(current) => current.max(value),
            None => value,
        });

        let delta = value - self.mean;
        self.mean += delta / self.count as f64;
        let delta2 = value - self.mean;
        self.m2 += delta * delta2;
    }
}

impl NumericStats {
    pub fn summarize(&self, include_percentiles: bool) -> Option<NumericSummary> {
        if self.count == 0 {
            return None;
        }

        let mut sorted = self.values.clone();
        sorted.sort_by(f64::total_cmp);

        let percentile_summary = include_percentiles.then(|| PercentileSummary {
            p5: percentile_from_sorted(&sorted, 0.05),
            p25: percentile_from_sorted(&sorted, 0.25),
            p75: percentile_from_sorted(&sorted, 0.75),
            p95: percentile_from_sorted(&sorted, 0.95),
        });

        Some(NumericSummary {
            analyzed_count: self.count,
            min: self.min.unwrap_or(0.0),
            max: self.max.unwrap_or(0.0),
            mean: self.mean,
            median: percentile_from_sorted(&sorted, 0.50),
            std_dev: if self.count > 1 {
                (self.m2 / (self.count as f64 - 1.0)).sqrt()
            } else {
                0.0
            },
            percentiles: percentile_summary,
            outlier_count: iqr_outlier_count(&sorted),
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct DateStats {
    count: u64,
    min: Option<NaiveDate>,
    max: Option<NaiveDate>,
}

impl Metric for DateStats {
    fn observe(&mut self, inspection: &ValueInspection<'_>) {
        let Some(value) = inspection.date_value() else {
            return;
        };

        self.count += 1;
        self.min = Some(match self.min {
            Some(current) => current.min(value),
            None => value,
        });
        self.max = Some(match self.max {
            Some(current) => current.max(value),
            None => value,
        });
    }
}

impl DateStats {
    pub fn summarize(&self) -> Option<DateSummary> {
        Some(DateSummary {
            analyzed_count: self.count,
            min: self.min?,
            max: self.max?,
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct TextStats {
    shortest_length: Option<usize>,
    longest_length: Option<usize>,
}

impl Metric for TextStats {
    fn observe(&mut self, inspection: &ValueInspection<'_>) {
        let length = inspection.raw().chars().count();
        self.shortest_length = Some(match self.shortest_length {
            Some(current) => current.min(length),
            None => length,
        });
        self.longest_length = Some(match self.longest_length {
            Some(current) => current.max(length),
            None => length,
        });
    }
}

impl TextStats {
    pub fn summarize(&self) -> Option<TextSummary> {
        Some(TextSummary {
            shortest_length: self.shortest_length?,
            longest_length: self.longest_length?,
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct FrequencyTable {
    counts: HashMap<String, u64>,
}

impl Metric for FrequencyTable {
    fn observe(&mut self, inspection: &ValueInspection<'_>) {
        *self.counts.entry(inspection.raw().to_string()).or_insert(0) += 1;
    }
}

impl FrequencyTable {
    pub fn unique_count(&self) -> u64 {
        self.counts.len() as u64
    }

    pub fn top_n(&self, limit: usize) -> Vec<FrequencyEntry> {
        let mut entries = self.entries();
        entries.sort_by(|left, right| {
            right
                .count
                .cmp(&left.count)
                .then_with(|| left.value.cmp(&right.value))
        });
        entries.truncate(limit);
        entries
    }

    pub fn least_n(&self, limit: usize) -> Vec<FrequencyEntry> {
        let mut entries = self.entries();
        entries.sort_by(|left, right| {
            left.count
                .cmp(&right.count)
                .then_with(|| left.value.cmp(&right.value))
        });
        entries.truncate(limit);
        entries
    }

    pub fn histogram_lines(&self, max_width: usize, limit: usize) -> Vec<String> {
        let entries = self.top_n(limit);
        let largest = entries.first().map(|entry| entry.count).unwrap_or(0);
        if largest == 0 {
            return Vec::new();
        }

        entries
            .into_iter()
            .map(|entry| {
                let width = ((entry.count as f64 / largest as f64) * max_width as f64)
                    .round()
                    .max(1.0) as usize;
                format!(
                    "{:<18} | {} ({})",
                    truncate_for_histogram(&entry.value),
                    "#".repeat(width),
                    entry.count
                )
            })
            .collect()
    }

    fn entries(&self) -> Vec<FrequencyEntry> {
        self.counts
            .iter()
            .map(|(value, count)| FrequencyEntry {
                value: value.clone(),
                count: *count,
            })
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct NumericSummary {
    pub analyzed_count: u64,
    pub min: f64,
    pub max: f64,
    pub mean: f64,
    pub median: f64,
    pub std_dev: f64,
    pub percentiles: Option<PercentileSummary>,
    pub outlier_count: usize,
}

#[derive(Debug, Clone)]
pub struct PercentileSummary {
    pub p5: f64,
    pub p25: f64,
    pub p75: f64,
    pub p95: f64,
}

#[derive(Debug, Clone)]
pub struct DateSummary {
    pub analyzed_count: u64,
    pub min: NaiveDate,
    pub max: NaiveDate,
}

#[derive(Debug, Clone)]
pub struct TextSummary {
    pub shortest_length: usize,
    pub longest_length: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrequencyEntry {
    pub value: String,
    pub count: u64,
}

pub fn is_low_cardinality(unique_count: u64, non_null_count: u64, threshold: usize) -> bool {
    if non_null_count == 0 {
        return false;
    }

    let unique_ratio = unique_count as f64 / non_null_count as f64;
    unique_count <= threshold as u64 && unique_ratio <= 0.80
}

fn percentile_from_sorted(sorted: &[f64], percentile: f64) -> f64 {
    if sorted.len() == 1 {
        return sorted[0];
    }

    let position = percentile.clamp(0.0, 1.0) * (sorted.len() - 1) as f64;
    let lower = position.floor() as usize;
    let upper = position.ceil() as usize;

    if lower == upper {
        sorted[lower]
    } else {
        let weight = position - lower as f64;
        sorted[lower] + (sorted[upper] - sorted[lower]) * weight
    }
}

fn iqr_outlier_count(sorted: &[f64]) -> usize {
    if sorted.len() < 4 {
        return 0;
    }

    let q1 = percentile_from_sorted(sorted, 0.25);
    let q3 = percentile_from_sorted(sorted, 0.75);
    let iqr = q3 - q1;

    if iqr == 0.0 {
        return 0;
    }

    let lower_bound = q1 - 1.5 * iqr;
    let upper_bound = q3 + 1.5 * iqr;
    sorted
        .iter()
        .filter(|value| **value < lower_bound || **value > upper_bound)
        .count()
}

fn join_kinds(kinds: &[PrimitiveKind]) -> String {
    kinds
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

fn truncate_for_histogram(value: &str) -> String {
    let mut output = value.replace('\n', "\\n");
    if output.chars().count() > 18 {
        output = output.chars().take(15).collect::<String>() + "...";
    }
    output
}

#[cfg(test)]
mod tests {
    use super::{FrequencyTable, Metric, NumericStats, TypeCounter};
    use crate::value::{InferredType, ValueInspection};

    #[test]
    fn infers_float_when_integers_and_floats_mix() {
        let mut counter = TypeCounter::default();
        for raw in ["10", "12.5", "13", "9.75"] {
            counter.observe(&ValueInspection::inspect(raw));
        }

        let inferred = counter.infer_type(4, 4, 20);
        assert_eq!(inferred, InferredType::Float);
        assert!(counter.mixed_type_warning(inferred, 4).is_some());
    }

    #[test]
    fn computes_numeric_summary_with_outlier_count() {
        let mut stats = NumericStats::default();
        for raw in ["10", "12", "12", "13", "999"] {
            stats.observe(&ValueInspection::inspect(raw));
        }

        let summary = stats.summarize(true).expect("numeric summary");
        assert_eq!(summary.analyzed_count, 5);
        assert_eq!(summary.min, 10.0);
        assert_eq!(summary.max, 999.0);
        assert!(summary.outlier_count >= 1);
        assert!(summary.percentiles.is_some());
    }

    #[test]
    fn ranks_frequencies() {
        let mut table = FrequencyTable::default();
        for raw in ["red", "blue", "red", "green", "red", "blue"] {
            table.observe(&ValueInspection::inspect(raw));
        }

        let top = table.top_n(2);
        assert_eq!(top[0].value, "red");
        assert_eq!(top[0].count, 3);
        assert_eq!(table.unique_count(), 3);
    }
}
