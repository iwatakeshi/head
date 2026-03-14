use clap::Parser;

/// Print the first 10 lines of each FILE to standard output.
///
/// With more than one FILE, precede each with a header giving the file name.
/// With no FILE, or when FILE is -, read standard input.
#[derive(Parser, Debug)]
#[command(
    name = "head",
    version,
    after_help = "\
NUM may have a multiplier suffix:
b 512, kB 1000, K 1024, MB 1000*1000, M 1024*1024,
GB 1000*1000*1000, G 1024*1024*1024, and so on for T, P, E, Z, Y, R, Q.
Binary prefixes can be used, too: KiB=K, MiB=M, and so on."
)]
pub struct Cli {
    /// Print the first NUM bytes of each file; with the leading '-', print all
    /// but the last NUM bytes of each file.
    #[arg(
        short = 'c',
        long,
        value_name = "[-]NUM",
        allow_hyphen_values = true,
        conflicts_with = "lines"
    )]
    pub bytes: Option<String>,

    /// Print the first NUM lines instead of the first 10; with the leading
    /// '-', print all but the last NUM lines of each file.
    #[arg(short = 'n', long, value_name = "[-]NUM", allow_hyphen_values = true)]
    pub lines: Option<String>,

    /// Never print headers giving file names.
    #[arg(short = 'q', long, visible_aliases = ["silent"], conflicts_with = "verbose")]
    pub quiet: bool,

    /// Always print headers giving file names.
    #[arg(short = 'v', long)]
    pub verbose: bool,

    /// Line delimiter is NUL, not newline.
    #[arg(short = 'z', long)]
    pub zero_terminated: bool,

    /// Files to read. Use `-` for standard input.
    #[arg(value_name = "FILE")]
    pub files: Vec<String>,
}

/// Transform the raw argument list so that the legacy `-NUM` shorthand
/// (e.g. `head -5 file`) is converted to `head -n 5 file` before clap sees
/// the arguments.
///
/// This also avoids accidentally rewriting the *value* of a `-n` or `-c`
/// option when that value itself starts with a digit (normal) or with `-`
/// followed by digits (the "all but last N" form).
pub fn preprocess_args(args: impl IntoIterator<Item = String>) -> Vec<String> {
    let mut args: Vec<String> = args.into_iter().collect();
    if args.is_empty() {
        return args;
    }

    // Keep the program name intact.
    let program = args.remove(0);
    let mut result = vec![program];

    let mut iter = args.into_iter().peekable();
    while let Some(arg) = iter.next() {
        // After `--` everything is a positional argument; pass through as-is.
        if arg == "--" {
            result.push(arg);
            result.extend(iter);
            break;
        }

        if let Some(rest) = arg.strip_prefix('-') {
            // `-NUM` shorthand → `-n NUM`
            if !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit()) {
                result.push("-n".to_string());
                result.push(rest.to_string());
                continue;
            }

            // `-n` or `-c` as a *separate* token: the *next* token is the
            // value, so skip preprocessing for it.
            if arg == "-n" || arg == "-c" {
                result.push(arg);
                if let Some(value) = iter.next() {
                    result.push(value);
                }
                continue;
            }
        }

        result.push(arg);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn preprocess(args: &[&str]) -> Vec<String> {
        preprocess_args(args.iter().map(|s| s.to_string()))
    }

    #[test]
    fn test_legacy_shorthand() {
        assert_eq!(
            preprocess(&["head", "-5", "file.txt"]),
            vec!["head", "-n", "5", "file.txt"]
        );
    }

    #[test]
    fn test_negative_value_not_preprocessed() {
        // `-n -5` should stay as-is so clap receives `-n` then `-5`
        assert_eq!(
            preprocess(&["head", "-n", "-5", "file.txt"]),
            vec!["head", "-n", "-5", "file.txt"]
        );
    }

    #[test]
    fn test_normal_n_value_not_preprocessed() {
        assert_eq!(
            preprocess(&["head", "-n", "3", "file.txt"]),
            vec!["head", "-n", "3", "file.txt"]
        );
    }

    #[test]
    fn test_end_of_options() {
        // After `--`, `-5` is treated as a filename, not a shorthand
        assert_eq!(preprocess(&["head", "--", "-5"]), vec!["head", "--", "-5"]);
    }

    #[test]
    fn test_non_numeric_flags_untouched() {
        assert_eq!(
            preprocess(&["head", "-v", "file.txt"]),
            vec!["head", "-v", "file.txt"]
        );
    }
}
