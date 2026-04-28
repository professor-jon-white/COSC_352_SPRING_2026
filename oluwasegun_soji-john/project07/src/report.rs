use std::fmt::Write;

use crate::profile::{ColumnReport, DatasetReport, ProfilerConfig};
use crate::stats::FrequencyEntry;
use crate::value::InferredType;

pub trait RenderableReport {
    fn render(&self, config: &ProfilerConfig) -> String;
}

impl RenderableReport for DatasetReport {
    fn render(&self, config: &ProfilerConfig) -> String {
        let mut output = String::new();

        writeln!(&mut output, "CSV Profile Report").unwrap();
        writeln!(&mut output, "==================").unwrap();
        writeln!(&mut output).unwrap();
        writeln!(&mut output, "Dataset Summary").unwrap();
        writeln!(&mut output, "---------------").unwrap();
        writeln!(&mut output, "Source: {}", self.source_name).unwrap();
        writeln!(&mut output, "Rows: {}", self.row_count).unwrap();
        writeln!(&mut output, "Columns: {}", self.column_count).unwrap();
        writeln!(&mut output, "Delimiter: {}", config.delimiter_label()).unwrap();
        writeln!(
            &mut output,
            "Percentiles: {}",
            if config.include_percentiles {
                "enabled"
            } else {
                "disabled"
            }
        )
        .unwrap();
        writeln!(
            &mut output,
            "Categorical histogram: {}",
            if config.include_histogram {
                "enabled"
            } else {
                "disabled"
            }
        )
        .unwrap();

        for (index, column) in self.columns.iter().enumerate() {
            writeln!(&mut output).unwrap();
            writeln!(&mut output, "{}", "-".repeat(70)).unwrap();
            writeln!(&mut output, "{}", column.render_with_index(index + 1)).unwrap();
        }

        output.trim_end().to_string()
    }
}

impl ColumnReport {
    fn render_with_index(&self, index: usize) -> String {
        let mut output = String::new();
        writeln!(&mut output, "Column {index}: {}", self.name).unwrap();
        writeln!(&mut output, "  Inferred type: {}", self.inferred_type).unwrap();
        writeln!(&mut output, "  Row count: {}", self.row_count).unwrap();
        writeln!(
            &mut output,
            "  Null count / %: {} ({:.2}%)",
            self.null_count, self.null_percentage
        )
        .unwrap();
        writeln!(&mut output, "  Non-null count: {}", self.non_null_count).unwrap();
        writeln!(&mut output, "  Unique value count: {}", self.unique_count).unwrap();
        writeln!(
            &mut output,
            "  Mixed-type warning: {}",
            self.mixed_type_warning.as_deref().unwrap_or("none")
        )
        .unwrap();
        writeln!(
            &mut output,
            "  Constant column warning: {}",
            yes_no(self.constant_column_warning)
        )
        .unwrap();
        writeln!(
            &mut output,
            "  Low-cardinality categorical warning: {}",
            yes_no(self.low_cardinality_warning)
        )
        .unwrap();

        if let Some(summary) = &self.numeric_summary {
            writeln!(
                &mut output,
                "  Values used for numeric stats: {}",
                summary.analyzed_count
            )
            .unwrap();
            writeln!(
                &mut output,
                "  Min / Max: {} / {}",
                format_number(summary.min),
                format_number(summary.max)
            )
            .unwrap();
            writeln!(
                &mut output,
                "  Mean / Median / Std dev: {} / {} / {}",
                format_number(summary.mean),
                format_number(summary.median),
                format_number(summary.std_dev)
            )
            .unwrap();
            writeln!(
                &mut output,
                "  Outliers (IQR rule): {}",
                summary.outlier_count
            )
            .unwrap();

            if let Some(percentiles) = &summary.percentiles {
                writeln!(
                    &mut output,
                    "  Percentiles: p5={}  p25={}  p75={}  p95={}",
                    format_number(percentiles.p5),
                    format_number(percentiles.p25),
                    format_number(percentiles.p75),
                    format_number(percentiles.p95)
                )
                .unwrap();
            }
        }

        if let Some(summary) = &self.date_summary {
            writeln!(
                &mut output,
                "  Values used for date stats: {}",
                summary.analyzed_count
            )
            .unwrap();
            writeln!(
                &mut output,
                "  Min / Max: {} / {}",
                summary.min, summary.max
            )
            .unwrap();
        }

        if matches!(
            self.inferred_type,
            InferredType::Text | InferredType::Categorical | InferredType::Boolean
        ) {
            if let Some(summary) = &self.text_summary {
                writeln!(
                    &mut output,
                    "  Shortest / longest string length: {} / {}",
                    summary.shortest_length, summary.longest_length
                )
                .unwrap();
            }
        }

        if !self.top_values.is_empty() {
            writeln!(&mut output, "  Top values:").unwrap();
            write_frequency_block(&mut output, &self.top_values).unwrap();
        }

        if !self.rare_values.is_empty() {
            writeln!(&mut output, "  Least frequent values:").unwrap();
            write_frequency_block(&mut output, &self.rare_values).unwrap();
        }

        if !self.histogram_lines.is_empty() {
            writeln!(&mut output, "  Histogram:").unwrap();
            for line in &self.histogram_lines {
                writeln!(&mut output, "    {line}").unwrap();
            }
        }

        output.trim_end().to_string()
    }
}

fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}

fn format_number(value: f64) -> String {
    if (value.fract()).abs() < 1e-10 {
        format!("{value:.0}")
    } else {
        format!("{value:.4}")
    }
}

fn write_frequency_block(output: &mut String, values: &[FrequencyEntry]) -> std::fmt::Result {
    for entry in values {
        writeln!(output, "    - {:?}: {}", entry.value, entry.count)?;
    }
    Ok(())
}
