# head

A cross-platform `head` clone written in Rust.

`head` is a command-line utility that outputs the first N lines (default 10) or bytes from standard input or one or more files. It supports all standard GNU `head` features including multiple file support with headers and negative count syntax.

> **See also:** [iwatakeshi/tail](https://github.com/iwatakeshi/tail) — a companion cross-platform `tail` clone written in Rust.

## Features

- **Line-based output** (`-n`): Output the first N lines (default 10)
- **Byte-based output** (`-c`): Output the first N bytes
- **Negative count** (`-NUM`): Use `-n -NUM` or `-c -NUM` to output all but the last NUM lines/bytes
- **Legacy shorthand**: `-NUM` is accepted as a shorthand for `-n NUM` (e.g. `head -5 file`)
- **Multiple files**: Process multiple files with automatic headers
- **Header control** (`-q`/`-v`): Suppress or force file name headers
- **Size suffixes**: Support for `K`, `M`, `G`, `kB`, `MB`, `GB`, etc.
- **Zero-terminated** (`-z`): Use NUL as line delimiter instead of newline
- **Cross-platform**: Works on Linux, macOS, and Windows

## Usage Examples

```bash
# Display first 10 lines of a file (default)
head /etc/passwd

# Display first 20 lines from stdin
cat /path/to/file | head -n 20

# Display first 5 lines of a file
head -n 5 /etc/passwd

# Legacy shorthand for first 5 lines
head -5 /etc/passwd

# Display first 100 bytes
head -c 100 myfile.txt

# Display all but the last 5 lines
head -n -5 myfile.txt

# Display all but the last 100 bytes
head -c -100 myfile.txt

# Multiple files with headers
head -n 5 file1.txt file2.txt file3.txt

# Use size suffixes
head -c 2K myfile.txt    # First 2 KiB (2048 bytes)
head -c 1M myfile.txt    # First 1 MiB (1048576 bytes)

# Suppress headers with multiple files
head -q -n 5 file1.txt file2.txt

# Use NUL-terminated lines
head -z -n 5 myfile.txt
```

## Full Usage

```
Print the first 10 lines of each FILE to standard output.
With more than one FILE, precede each with a header giving the file name.

With no FILE, or when FILE is -, read standard input.

Usage: head [OPTIONS] [FILE]...

Arguments:
  [FILE]...  Files to read from (optional, reads stdin if omitted or -)

Options:
  -c, --bytes <[-]NUM>      Print the first NUM bytes of each file; with the leading '-',
                            print all but the last NUM bytes of each file
  -n, --lines <[-]NUM>      Print the first NUM lines instead of the first 10; with the
                            leading '-', print all but the last NUM lines of each file
  -q, --quiet               Never print headers giving file names
  -v, --verbose             Always print headers giving file names
  -z, --zero-terminated     Line delimiter is NUL, not newline
  -h, --help                Print help
  -V, --version             Print version
```

## Size Suffixes

NUM may have a multiplier suffix:
- `b` = 512
- `kB` = 1000, `K` / `KiB` = 1024
- `MB` = 1,000,000, `M` / `MiB` = 1,048,576
- `GB` = 1,000,000,000, `G` / `GiB` = 1,073,741,824
- `TB` = 1,000,000,000,000, `T` / `TiB` = 1,099,511,627,776

## Building

```bash
# Build release binary
cargo build --release

# Run tests
cargo test

# Run linter
cargo clippy

# Format code
cargo fmt
```

## License

MIT