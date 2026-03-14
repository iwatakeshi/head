use std::collections::VecDeque;
use std::io::{self, BufRead, BufReader, Read, Write};

use crate::config::Count;
use crate::error::HeadError;

/// Internal result alias.
pub type Result<T> = std::result::Result<T, HeadError>;

// Size of the read/write buffer used by streaming operations.
const CHUNK_SIZE: usize = 8_192;

// ── Public trait ────────────────────────────────────────────────────────────

/// Strategy for extracting a prefix (or suffix-excluding portion) of a byte
/// stream.
pub trait Processor {
    fn process(&self, input: &mut dyn Read, output: &mut dyn Write) -> Result<()>;
}

// ── Concrete processors ─────────────────────────────────────────────────────

/// Line-oriented processor: counts records delimited by `delimiter`.
pub struct LineProcessor {
    count: Count,
    delimiter: u8,
}

/// Byte-oriented processor: counts raw bytes.
pub struct ByteProcessor {
    count: Count,
}

impl LineProcessor {
    pub fn new(count: Count, delimiter: u8) -> Self {
        Self { count, delimiter }
    }
}

impl ByteProcessor {
    pub fn new(count: Count) -> Self {
        Self { count }
    }
}

impl Processor for LineProcessor {
    fn process(&self, input: &mut dyn Read, output: &mut dyn Write) -> Result<()> {
        match self.count {
            Count::First(n) => first_n_lines(input, output, n, self.delimiter),
            Count::AllButLast(n) => all_but_last_n_lines(input, output, n, self.delimiter),
        }
    }
}

impl Processor for ByteProcessor {
    fn process(&self, input: &mut dyn Read, output: &mut dyn Write) -> Result<()> {
        match self.count {
            Count::First(n) => first_n_bytes(input, output, n),
            Count::AllButLast(n) => all_but_last_n_bytes(input, output, n),
        }
    }
}

// ── Line algorithms ─────────────────────────────────────────────────────────

/// Output the first `n` delimiter-terminated records.
fn first_n_lines(
    input: &mut dyn Read,
    output: &mut dyn Write,
    n: u64,
    delimiter: u8,
) -> Result<()> {
    if n == 0 {
        return Ok(());
    }

    let mut reader = BufReader::new(input);
    let mut buf = Vec::with_capacity(CHUNK_SIZE);
    let mut count = 0u64;

    while count < n {
        buf.clear();
        let bytes_read = reader.read_until(delimiter, &mut buf)?;
        if bytes_read == 0 {
            break;
        }
        output.write_all(&buf)?;
        // Count every chunk returned by read_until as one record, regardless
        // of whether the final byte is the delimiter (handles unterminated
        // last lines correctly).
        count += 1;
    }

    Ok(())
}

/// Output all records except the last `n`.
///
/// Uses a ring buffer of at most `n` records so that only `O(n)` lines need
/// to be held in memory at any time.
fn all_but_last_n_lines(
    input: &mut dyn Read,
    output: &mut dyn Write,
    n: u64,
    delimiter: u8,
) -> Result<()> {
    if n == 0 {
        io::copy(input, output)?;
        return Ok(());
    }

    // Cap ring capacity to usize::MAX to avoid impossible allocations.
    // For realistic workloads n fits in usize.
    let n = n.min(usize::MAX as u64) as usize;

    let mut reader = BufReader::new(input);
    let mut buf = Vec::with_capacity(CHUNK_SIZE);
    // Ring buffer holding the last `n` lines seen so far.
    let mut ring: VecDeque<Vec<u8>> = VecDeque::new();

    loop {
        buf.clear();
        let bytes_read = reader.read_until(delimiter, &mut buf)?;
        if bytes_read == 0 {
            break;
        }

        if ring.len() >= n {
            // The oldest entry is now safe to emit — it won't be one of the
            // last `n` lines.
            let oldest = ring.pop_front().unwrap();
            output.write_all(&oldest)?;
        }
        ring.push_back(buf.clone());
    }
    // Remaining entries in `ring` are the last ≤ n lines; discard them.

    Ok(())
}

// ── Byte algorithms ─────────────────────────────────────────────────────────

/// Output the first `n` bytes.
fn first_n_bytes(input: &mut dyn Read, output: &mut dyn Write, n: u64) -> Result<()> {
    let mut limited = input.take(n);
    io::copy(&mut limited, output)?;
    Ok(())
}

/// Output all bytes except the last `n`.
///
/// Maintains a ring buffer of `n` bytes and flushes bytes that leave the
/// window in chunks for efficiency.
fn all_but_last_n_bytes(input: &mut dyn Read, output: &mut dyn Write, n: u64) -> Result<()> {
    if n == 0 {
        io::copy(input, output)?;
        return Ok(());
    }

    // For values of n that exceed usize::MAX (only possible on 32-bit targets)
    // fall back to reading all content into memory.
    if n > usize::MAX as u64 {
        let mut data = Vec::new();
        input.read_to_end(&mut data)?;
        let len = data.len() as u64;
        if len > n {
            output.write_all(&data[..(len - n) as usize])?;
        }
        return Ok(());
    }

    let n = n as usize;
    let mut ring: VecDeque<u8> = VecDeque::new();
    let mut read_buf = vec![0u8; CHUNK_SIZE];
    let mut out_buf = Vec::with_capacity(CHUNK_SIZE);

    loop {
        let bytes_read = input.read(&mut read_buf)?;
        if bytes_read == 0 {
            break;
        }

        for &b in &read_buf[..bytes_read] {
            ring.push_back(b);
            if ring.len() > n {
                // This byte has left the "last n" window; safe to emit.
                out_buf.push(ring.pop_front().unwrap());
                if out_buf.len() >= CHUNK_SIZE {
                    output.write_all(&out_buf)?;
                    out_buf.clear();
                }
            }
        }
    }

    if !out_buf.is_empty() {
        output.write_all(&out_buf)?;
    }

    Ok(())
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn run_line(count: Count, delimiter: u8, input: &[u8]) -> Vec<u8> {
        let proc = LineProcessor::new(count, delimiter);
        let mut out = Vec::new();
        proc.process(&mut Cursor::new(input), &mut out).unwrap();
        out
    }

    fn run_byte(count: Count, input: &[u8]) -> Vec<u8> {
        let proc = ByteProcessor::new(count);
        let mut out = Vec::new();
        proc.process(&mut Cursor::new(input), &mut out).unwrap();
        out
    }

    // ── first_n_lines ───────────────────────────────────────────────────────

    #[test]
    fn first_n_lines_basic() {
        let input = b"alpha\nbeta\ngamma\n";
        assert_eq!(run_line(Count::First(2), b'\n', input), b"alpha\nbeta\n");
    }

    #[test]
    fn first_n_lines_zero() {
        assert_eq!(run_line(Count::First(0), b'\n', b"hello\n"), b"");
    }

    #[test]
    fn first_n_lines_more_than_exist() {
        let input = b"a\nb\n";
        assert_eq!(run_line(Count::First(10), b'\n', input), b"a\nb\n");
    }

    #[test]
    fn first_n_lines_no_trailing_newline() {
        // The last "line" without a trailing delimiter still counts.
        let input = b"a\nb\nc";
        assert_eq!(run_line(Count::First(2), b'\n', input), b"a\nb\n");
        assert_eq!(run_line(Count::First(3), b'\n', input), b"a\nb\nc");
    }

    #[test]
    fn first_n_lines_nul_delimiter() {
        let input = b"a\0b\0c\0";
        assert_eq!(run_line(Count::First(2), b'\0', input), b"a\0b\0");
    }

    // ── all_but_last_n_lines ────────────────────────────────────────────────

    #[test]
    fn all_but_last_n_lines_basic() {
        let input = b"a\nb\nc\nd\ne\n";
        assert_eq!(run_line(Count::AllButLast(2), b'\n', input), b"a\nb\nc\n");
    }

    #[test]
    fn all_but_last_n_lines_zero() {
        let input = b"a\nb\n";
        assert_eq!(run_line(Count::AllButLast(0), b'\n', input), b"a\nb\n");
    }

    #[test]
    fn all_but_last_n_exceeds_line_count() {
        // Asking to skip more lines than exist → empty output.
        let input = b"a\nb\nc\n";
        assert_eq!(run_line(Count::AllButLast(10), b'\n', input), b"");
    }

    #[test]
    fn all_but_last_n_lines_exact() {
        let input = b"a\nb\nc\n";
        assert_eq!(run_line(Count::AllButLast(3), b'\n', input), b"");
        assert_eq!(run_line(Count::AllButLast(2), b'\n', input), b"a\n");
        assert_eq!(run_line(Count::AllButLast(1), b'\n', input), b"a\nb\n");
    }

    // ── first_n_bytes ───────────────────────────────────────────────────────

    #[test]
    fn first_n_bytes_basic() {
        assert_eq!(run_byte(Count::First(3), b"hello"), b"hel");
    }

    #[test]
    fn first_n_bytes_zero() {
        assert_eq!(run_byte(Count::First(0), b"hello"), b"");
    }

    #[test]
    fn first_n_bytes_more_than_exist() {
        assert_eq!(run_byte(Count::First(100), b"hello"), b"hello");
    }

    // ── all_but_last_n_bytes ────────────────────────────────────────────────

    #[test]
    fn all_but_last_n_bytes_basic() {
        assert_eq!(run_byte(Count::AllButLast(3), b"hello"), b"he");
    }

    #[test]
    fn all_but_last_n_bytes_zero() {
        assert_eq!(run_byte(Count::AllButLast(0), b"hello"), b"hello");
    }

    #[test]
    fn all_but_last_n_bytes_exceeds() {
        assert_eq!(run_byte(Count::AllButLast(10), b"hi"), b"");
    }

    #[test]
    fn all_but_last_n_bytes_exact() {
        assert_eq!(run_byte(Count::AllButLast(5), b"hello"), b"");
        assert_eq!(run_byte(Count::AllButLast(3), b"hello"), b"he");
        assert_eq!(run_byte(Count::AllButLast(1), b"hello"), b"hell");
    }
}
