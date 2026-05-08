use std::collections::HashSet;
use std::io::Cursor;
use std::slice;
use std::sync::atomic::{AtomicUsize, Ordering};

use serde::{Deserialize, Serialize};

use crate::profiler::{Profiler, ProfilerConfig};
use crate::report;
use crate::types::{DatasetReport, OutputFormat};

const DEFAULT_NULLS: &[&str] = &["", "null", "na", "n/a", "none"];

static LAST_OUTPUT_LEN: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct WasmProfileOptions {
    output_format: Option<WasmOutputFormat>,
    delimiter: Option<String>,
    has_headers: Option<bool>,
    percentiles: Option<Vec<f64>>,
    top_k: Option<usize>,
    top_k_capacity: Option<usize>,
    distinct_capacity: Option<usize>,
    sample_size: Option<usize>,
    null_values: Option<Vec<String>>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum WasmOutputFormat {
    Markdown,
    Json,
}

impl WasmOutputFormat {
    fn as_output_format(self) -> OutputFormat {
        match self {
            Self::Markdown => OutputFormat::Markdown,
            Self::Json => OutputFormat::Json,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Markdown => "markdown",
            Self::Json => "json",
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct WasmProfileSuccess {
    ok: bool,
    output_format: &'static str,
    rendered: String,
    report: DatasetReport,
}

#[derive(Debug, Serialize)]
struct WasmProfileError {
    ok: bool,
    error: String,
}

#[no_mangle]
pub extern "C" fn csvprof_alloc(len: usize) -> *mut u8 {
    let mut buffer = Vec::<u8>::with_capacity(len);
    let ptr = buffer.as_mut_ptr();
    std::mem::forget(buffer);
    ptr
}

#[no_mangle]
pub unsafe extern "C" fn csvprof_free(ptr: *mut u8, len: usize) {
    if ptr.is_null() {
        return;
    }
    drop(Vec::from_raw_parts(ptr, 0, len));
}

#[no_mangle]
pub unsafe extern "C" fn csvprof_profile(
    input_ptr: *const u8,
    input_len: usize,
    options_ptr: *const u8,
    options_len: usize,
) -> *mut u8 {
    let response = match borrowed_bytes(input_ptr, input_len)
        .and_then(|input| borrowed_bytes(options_ptr, options_len).map(|options| (input, options)))
        .and_then(|(input, options)| profile(input, options))
    {
        Ok(success) => serialize_json(&success),
        Err(error) => serialize_json(&WasmProfileError { ok: false, error }),
    };

    leak_response(response)
}

#[no_mangle]
pub extern "C" fn csvprof_last_output_len() -> usize {
    LAST_OUTPUT_LEN.load(Ordering::Relaxed)
}

unsafe fn borrowed_bytes<'a>(ptr: *const u8, len: usize) -> Result<&'a [u8], String> {
    if len == 0 {
        return Ok(&[]);
    }
    if ptr.is_null() {
        return Err("received a null pointer for a non-empty buffer".to_string());
    }
    Ok(slice::from_raw_parts(ptr, len))
}

fn profile(input: &[u8], options_bytes: &[u8]) -> Result<WasmProfileSuccess, String> {
    let options = parse_options(options_bytes)?;
    let output_format = options.output_format.unwrap_or(WasmOutputFormat::Markdown);

    let mut profiler = Profiler::new(options.profiler_config()?);
    profiler
        .profile_reader(Cursor::new(input))
        .map_err(|error| error.to_string())?;

    let report = profiler.finalize();
    let rendered = report::render(&report, output_format.as_output_format())
        .map_err(|error| error.to_string())?;

    Ok(WasmProfileSuccess {
        ok: true,
        output_format: output_format.as_str(),
        rendered,
        report,
    })
}

fn parse_options(options_bytes: &[u8]) -> Result<WasmProfileOptions, String> {
    if options_bytes.iter().all(u8::is_ascii_whitespace) {
        return Ok(WasmProfileOptions::default());
    }
    serde_json::from_slice(options_bytes).map_err(|error| format!("invalid options JSON: {error}"))
}

impl WasmProfileOptions {
    fn profiler_config(&self) -> Result<ProfilerConfig, String> {
        let top_k = self.top_k.unwrap_or(5).max(1);

        Ok(ProfilerConfig {
            delimiter: self.delimiter_byte()?,
            has_headers: self.has_headers.unwrap_or(true),
            top_k,
            top_k_capacity: self.top_k_capacity.unwrap_or(32).max(top_k),
            distinct_capacity: self.distinct_capacity.unwrap_or(1024).max(32),
            sample_size: self.sample_size.unwrap_or(4096).max(32),
            percentiles: self.percentiles()?,
            null_values: self.null_values(),
        })
    }

    fn delimiter_byte(&self) -> Result<u8, String> {
        let delimiter = self.delimiter.as_deref().unwrap_or(",");
        let mut chars = delimiter.chars();
        let Some(delimiter) = chars.next() else {
            return Err("delimiter cannot be empty".to_string());
        };
        if chars.next().is_some() {
            return Err(format!(
                "delimiter must be a single character, got `{delimiter}`"
            ));
        }
        u8::try_from(delimiter as u32)
            .map_err(|_| format!("delimiter must be a single-byte character, got `{delimiter}`"))
    }

    fn percentiles(&self) -> Result<Vec<f64>, String> {
        let percentiles = self.percentiles.clone().unwrap_or_default();
        for percentile in &percentiles {
            if !(0.0..=100.0).contains(percentile) {
                return Err(format!(
                    "invalid percentile `{percentile}`; expected a number from 0 to 100"
                ));
            }
        }
        Ok(percentiles)
    }

    fn null_values(&self) -> HashSet<String> {
        let mut null_values: HashSet<String> = DEFAULT_NULLS
            .iter()
            .map(|value| value.to_string())
            .collect();

        if let Some(custom_values) = &self.null_values {
            for value in custom_values {
                null_values.insert(value.trim().to_ascii_lowercase());
            }
        }

        null_values
    }
}

fn serialize_json<T: Serialize>(value: &T) -> Vec<u8> {
    serde_json::to_vec(value).unwrap_or_else(|error| {
        format!(
            r#"{{"ok":false,"error":"failed to serialize WASM response: {}"}}"#,
            escape_json_string(&error.to_string())
        )
        .into_bytes()
    })
}

fn leak_response(bytes: Vec<u8>) -> *mut u8 {
    let len = bytes.len();
    let mut bytes = bytes.into_boxed_slice();
    let ptr = bytes.as_mut_ptr();
    LAST_OUTPUT_LEN.store(len, Ordering::Relaxed);
    std::mem::forget(bytes);
    ptr
}

fn escape_json_string(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}
