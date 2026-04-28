use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CsvProfError {
    #[error("failed to open {path}: {source}")]
    OpenFile { path: String, source: io::Error },

    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("CSV parse error: {0}")]
    Csv(#[from] csv::Error),
}
