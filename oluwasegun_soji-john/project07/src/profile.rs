use std::collections::HashSet;
use std::io::Read;

use csv::{ReaderBuilder, StringRecord};

use crate::error::CsvProfError;
use crate::stats::{
    DateStats, DateSummary, FrequencyEntry, FrequencyTable, Metric, NumericStats, NumericSummary,
    TextStats, TextSummary, TypeCounter, is_low_cardinality,
};
use crate::value::{InferredType, ValueInspection};

#[derive(Debug, Clone)]
pub struct ProfilerConfig {
    pub delimiter: u8,
    pub has_headers: bool,
    pub include_percentiles: bool,
    pub include_histogram: bool,
    pub categorical_threshold: usize,
    pub max_frequencies: usize,
    pub null_markers: HashSet<String>,
}

impl ProfilerConfig {
    pub fn is_null(&self, raw: &str) -> bool {
        let trimmed = raw.trim();
        trimmed.is_empty() || self.null_markers.contains(&trimmed.to_ascii_lowercase())
    }

    pub fn delimiter_label(&self) -> String {
        match self.delimiter {
            b'\t' => "\\t".to_string(),
            byte => (byte as char).to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DatasetReport {
    pub source_name: String,
    pub row_count: u64,
    pub column_count: usize,
    pub columns: Vec<ColumnReport>,
}

#[derive(Debug, Clone)]
pub struct ColumnReport {
    pub name: String,
    pub inferred_type: InferredType,
    pub row_count: u64,
    pub non_null_count: u64,
    pub null_count: u64,
    pub null_percentage: f64,
    pub unique_count: u64,
    pub mixed_type_warning: Option<String>,
    pub constant_column_warning: bool,
    pub low_cardinality_warning: bool,
    pub numeric_summary: Option<NumericSummary>,
    pub date_summary: Option<DateSummary>,
    pub text_summary: Option<TextSummary>,
    pub top_values: Vec<FrequencyEntry>,
    pub rare_values: Vec<FrequencyEntry>,
    pub histogram_lines: Vec<String>,
}

pub fn profile_csv<R: Read>(
    reader: R,
    source_name: impl Into<String>,
    config: &ProfilerConfig,
) -> Result<DatasetReport, CsvProfError> {
    let mut csv_reader = ReaderBuilder::new()
        .delimiter(config.delimiter)
        .has_headers(config.has_headers)
        .flexible(true)
        .from_reader(reader);

    let headers = if config.has_headers {
        csv_reader
            .headers()?
            .iter()
            .enumerate()
            .map(|(index, header)| sanitize_header(header, index))
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    let mut profiler = DatasetProfiler::new(source_name.into(), config.clone(), headers);

    for result in csv_reader.records() {
        let record = result?;
        profiler.process_record(&record);
    }

    Ok(profiler.finalize())
}

#[derive(Debug)]
struct DatasetProfiler {
    source_name: String,
    config: ProfilerConfig,
    row_count: u64,
    columns: Vec<ColumnProfiler>,
}

impl DatasetProfiler {
    fn new(source_name: String, config: ProfilerConfig, headers: Vec<String>) -> Self {
        let columns = headers
            .into_iter()
            .map(|header| ColumnProfiler::new(header, 0))
            .collect();

        Self {
            source_name,
            config,
            row_count: 0,
            columns,
        }
    }

    fn process_record(&mut self, record: &StringRecord) {
        if record.len() > self.columns.len() {
            self.extend_columns(record.len());
        } else if self.columns.is_empty() {
            self.extend_columns(record.len());
        }

        for index in 0..self.columns.len() {
            let value = record.get(index);
            self.columns[index].observe(value, &self.config);
        }

        self.row_count += 1;
    }

    fn extend_columns(&mut self, target_len: usize) {
        while self.columns.len() < target_len {
            let index = self.columns.len();
            let name = format!("column_{}", index + 1);
            self.columns.push(ColumnProfiler::new(name, self.row_count));
        }
    }

    fn finalize(self) -> DatasetReport {
        let columns = self
            .columns
            .into_iter()
            .map(|column| column.finalize(&self.config))
            .collect::<Vec<_>>();

        DatasetReport {
            source_name: self.source_name,
            row_count: self.row_count,
            column_count: columns.len(),
            columns,
        }
    }
}

#[derive(Debug)]
struct ColumnProfiler {
    name: String,
    row_count: u64,
    null_count: u64,
    types: TypeCounter,
    numeric: NumericStats,
    dates: DateStats,
    text: TextStats,
    frequencies: FrequencyTable,
}

impl ColumnProfiler {
    fn new(name: String, prior_rows: u64) -> Self {
        Self {
            name,
            row_count: prior_rows,
            null_count: prior_rows,
            types: TypeCounter::default(),
            numeric: NumericStats::default(),
            dates: DateStats::default(),
            text: TextStats::default(),
            frequencies: FrequencyTable::default(),
        }
    }

    fn observe(&mut self, raw_value: Option<&str>, config: &ProfilerConfig) {
        self.row_count += 1;

        let Some(raw_value) = raw_value else {
            self.null_count += 1;
            return;
        };

        let trimmed = raw_value.trim();
        if config.is_null(trimmed) {
            self.null_count += 1;
            return;
        }

        let inspection = ValueInspection::inspect(trimmed);
        self.types.observe(&inspection);
        self.numeric.observe(&inspection);
        self.dates.observe(&inspection);
        self.text.observe(&inspection);
        self.frequencies.observe(&inspection);
    }

    fn finalize(self, config: &ProfilerConfig) -> ColumnReport {
        let non_null_count = self.row_count.saturating_sub(self.null_count);
        let unique_count = self.frequencies.unique_count();
        let inferred_type =
            self.types
                .infer_type(non_null_count, unique_count, config.categorical_threshold);
        let mixed_type_warning = self.types.mixed_type_warning(inferred_type, non_null_count);
        let constant_column_warning = non_null_count > 0 && unique_count <= 1;
        let low_cardinality_warning = inferred_type == InferredType::Categorical
            && is_low_cardinality(unique_count, non_null_count, config.categorical_threshold);

        let numeric_summary = matches!(inferred_type, InferredType::Integer | InferredType::Float)
            .then(|| self.numeric.summarize(config.include_percentiles))
            .flatten();

        let date_summary = (inferred_type == InferredType::Date)
            .then(|| self.dates.summarize())
            .flatten();

        let text_summary = matches!(
            inferred_type,
            InferredType::Text | InferredType::Categorical | InferredType::Boolean
        )
        .then(|| self.text.summarize())
        .flatten();

        let (top_values, rare_values, histogram_lines) = if matches!(
            inferred_type,
            InferredType::Categorical | InferredType::Boolean
        ) {
            (
                self.frequencies.top_n(config.max_frequencies),
                self.frequencies.least_n(config.max_frequencies),
                if inferred_type == InferredType::Categorical && config.include_histogram {
                    self.frequencies.histogram_lines(32, config.max_frequencies)
                } else {
                    Vec::new()
                },
            )
        } else {
            (Vec::new(), Vec::new(), Vec::new())
        };

        ColumnReport {
            name: self.name,
            inferred_type,
            row_count: self.row_count,
            non_null_count,
            null_count: self.null_count,
            null_percentage: if self.row_count == 0 {
                0.0
            } else {
                self.null_count as f64 / self.row_count as f64 * 100.0
            },
            unique_count,
            mixed_type_warning,
            constant_column_warning,
            low_cardinality_warning,
            numeric_summary,
            date_summary,
            text_summary,
            top_values,
            rare_values,
            histogram_lines,
        }
    }
}

fn sanitize_header(header: &str, index: usize) -> String {
    let trimmed = header.trim();
    if trimmed.is_empty() {
        format!("column_{}", index + 1)
    } else {
        trimmed.to_string()
    }
}

#[cfg(test)]
mod tests {
    use crate::profile::{ProfilerConfig, profile_csv};
    use crate::report::RenderableReport;
    use crate::value::InferredType;

    fn config() -> ProfilerConfig {
        ProfilerConfig {
            delimiter: b',',
            has_headers: true,
            include_percentiles: true,
            include_histogram: true,
            categorical_threshold: 10,
            max_frequencies: 5,
            null_markers: ["na".to_string(), "null".to_string()].into_iter().collect(),
        }
    }

    #[test]
    fn profiles_mixed_csv_data() {
        let data = "\
id,score,active,joined,team,notes,mixed
1,10.5,true,2026-01-01,red,short,100
2,11.0,false,2026-01-02,blue,medium,200
3,12.0,yes,2026-01-03,red,longer note,300
4,13.5,no,2026-01-04,green,this one is much longer than the others,oops
";

        let report = profile_csv(data.as_bytes(), "inline.csv", &config()).expect("report");
        assert_eq!(report.row_count, 4);
        assert_eq!(report.column_count, 7);

        assert_eq!(report.columns[0].inferred_type, InferredType::Integer);
        assert_eq!(report.columns[1].inferred_type, InferredType::Float);
        assert_eq!(report.columns[2].inferred_type, InferredType::Boolean);
        assert_eq!(report.columns[3].inferred_type, InferredType::Date);
        assert_eq!(report.columns[4].inferred_type, InferredType::Categorical);
        assert_eq!(report.columns[5].inferred_type, InferredType::Text);
        assert_eq!(report.columns[6].inferred_type, InferredType::Text);

        let rendered = report.render(&config());
        assert!(rendered.contains("Dataset Summary"));
        assert!(rendered.contains("Column 2: score"));
        assert!(rendered.contains("Histogram"));
    }
}
