use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;

/// Format an [`io::Error`] the same way GNU coreutils does: the human-readable
/// message without the Rust `(os error N)` annotation.
fn fmt_io_err(e: &io::Error) -> String {
    let s = e.to_string();
    // Rust's io::Error includes " (os error N)" on most platforms; strip it.
    if let Some(pos) = s.rfind(" (os error ") {
        s[..pos].to_string()
    } else {
        s
    }
}

use clap::Parser;

pub mod cli;
pub mod config;
pub mod error;
pub mod processor;
pub mod size;

use cli::{preprocess_args, Cli};
use config::{Count, HeaderMode, HeadConfig, InputSource, OutputMode};
use error::HeadError;
use processor::{ByteProcessor, LineProcessor, Processor};

/// Build the appropriate [`Processor`] from a fully resolved [`HeadConfig`].
fn build_processor(config: &HeadConfig) -> Box<dyn Processor> {
    let delimiter = if config.zero_terminated { b'\0' } else { b'\n' };
    match config.mode {
        OutputMode::Lines(count) => Box::new(LineProcessor::new(count, delimiter)),
        OutputMode::Bytes(count) => Box::new(ByteProcessor::new(count)),
    }
}

/// Convert the parsed CLI arguments into a [`HeadConfig`].
fn config_from_cli(cli: Cli) -> Result<HeadConfig, HeadError> {
    let mode = match (cli.bytes, cli.lines) {
        (Some(_), Some(_)) => unreachable!("clap enforces --bytes / --lines conflict"),
        (Some(s), None) => OutputMode::Bytes(config::parse_count(&s, "bytes")?),
        (None, Some(s)) => OutputMode::Lines(config::parse_count(&s, "lines")?),
        (None, None) => OutputMode::Lines(Count::First(10)),
    };

    let header_mode = if cli.quiet {
        HeaderMode::Never
    } else if cli.verbose {
        HeaderMode::Always
    } else {
        HeaderMode::Auto
    };

    let sources: Vec<InputSource> = if cli.files.is_empty() {
        vec![InputSource::Stdin]
    } else {
        cli.files
            .into_iter()
            .map(|f| {
                if f == "-" {
                    InputSource::Stdin
                } else {
                    InputSource::File(PathBuf::from(f))
                }
            })
            .collect()
    };

    Ok(HeadConfig {
        mode,
        header_mode,
        zero_terminated: cli.zero_terminated,
        sources,
    })
}

/// Main entry point for the `head` binary.  Returns the process exit code.
pub fn run() -> i32 {
    let args = preprocess_args(std::env::args());
    let cli = Cli::parse_from(args);

    let config = match config_from_cli(cli) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("head: {e}");
            return 1;
        }
    };

    let processor = build_processor(&config);
    let num_sources = config.sources.len();
    let print_header = config.should_print_header(num_sources);

    let stdout = io::stdout();
    let mut output = stdout.lock();
    let mut had_error = false;

    for (idx, source) in config.sources.iter().enumerate() {
        if print_header {
            // GNU head prints a blank line *before* every header except the
            // first (the blank line is always emitted — it does not depend on
            // the previous file ending with a newline).
            if idx > 0 {
                if let Err(e) = output.write_all(b"\n") {
                    eprintln!("head: write error: {e}");
                    return 1;
                }
            }
            if let Err(e) = writeln!(output, "==> {} <==", source.display_name()) {
                eprintln!("head: write error: {e}");
                return 1;
            }
        }

        match source {
            InputSource::Stdin => {
                let stdin = io::stdin();
                let mut input = stdin.lock();
                if let Err(e) = processor.process(&mut input, &mut output) {
                    eprintln!("head: standard input: {e}");
                    had_error = true;
                }
            }
            InputSource::File(path) => {
                match File::open(path) {
                    Ok(mut file) => {
                        if let Err(e) = processor.process(&mut file, &mut output) {
                            eprintln!("head: {}: {e}", path.display());
                            had_error = true;
                        }
                    }
                    Err(e) => {
                        eprintln!(
                            "head: cannot open '{}' for reading: {}",
                            path.display(),
                            fmt_io_err(&e)
                        );
                        had_error = true;
                    }
                }
            }
        }
    }

    if had_error { 1 } else { 0 }
}
