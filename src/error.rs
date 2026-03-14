use std::io;
use thiserror::Error;

/// Errors that can occur during `head` processing.
#[derive(Debug, Error)]
pub enum HeadError {
    /// A numeric argument could not be parsed.
    #[error("invalid number of {kind}: {reason}")]
    InvalidCount { kind: String, reason: String },

    /// An I/O error occurred while reading a file.
    #[error("error reading '{path}': {source}")]
    FileRead { path: String, source: io::Error },

    /// An I/O error occurred while writing output.
    #[error("write error: {0}")]
    Write(#[from] io::Error),

    /// A file could not be opened.
    #[error("cannot open '{path}' for reading: {source}")]
    FileOpen { path: String, source: io::Error },
}
