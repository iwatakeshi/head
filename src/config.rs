use std::path::PathBuf;

use crate::error::HeadError;
use crate::size::parse_size;

/// How many lines or bytes to output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Count {
    /// Output the first `n` lines/bytes.
    First(u64),
    /// Output everything except the last `n` lines/bytes.
    AllButLast(u64),
}

/// What to measure when limiting output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputMode {
    Lines(Count),
    Bytes(Count),
}

/// Controls whether file-name headers are emitted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeaderMode {
    /// Emit headers only when more than one input source is present (default).
    Auto,
    /// Always emit headers (`-v` / `--verbose`).
    Always,
    /// Never emit headers (`-q` / `--quiet` / `--silent`).
    Never,
}

/// A single input source passed on the command line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputSource {
    /// Standard input (the literal `-` argument or an empty file list).
    Stdin,
    /// A regular file identified by path.
    File(PathBuf),
}

impl InputSource {
    /// The name shown in the header line (`==> name <==`).
    pub fn display_name(&self) -> &str {
        match self {
            InputSource::Stdin => "standard input",
            InputSource::File(p) => p.to_str().unwrap_or("<invalid utf-8>"),
        }
    }
}

/// Fully resolved configuration for a single `head` invocation.
#[derive(Debug)]
pub struct HeadConfig {
    pub mode: OutputMode,
    pub header_mode: HeaderMode,
    /// When `true` the line delimiter is NUL (`\0`) instead of newline.
    pub zero_terminated: bool,
    pub sources: Vec<InputSource>,
}

impl HeadConfig {
    /// Returns `true` when headers should be printed for the given number of
    /// input sources.
    pub fn should_print_header(&self, num_sources: usize) -> bool {
        match self.header_mode {
            HeaderMode::Auto => num_sources > 1,
            HeaderMode::Always => true,
            HeaderMode::Never => false,
        }
    }
}

/// Parse a `[-]NUM[SUFFIX]` string into a [`Count`].
///
/// A leading `-` means "all but the last N"; without it we take the first N.
pub fn parse_count(s: &str, kind: &str) -> Result<Count, HeadError> {
    let (all_but_last, num_str) = match s.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, s),
    };

    let n = parse_size(num_str).map_err(|reason| HeadError::InvalidCount {
        kind: kind.to_string(),
        reason,
    })?;

    if all_but_last {
        Ok(Count::AllButLast(n))
    } else {
        Ok(Count::First(n))
    }
}
