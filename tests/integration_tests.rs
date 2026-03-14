/// Integration tests for the `head` binary.
///
/// These tests exercise the binary end-to-end through `assert_cmd` and cover
/// every flag and edge-case described in the GNU coreutils `head` manual.
use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;
use tempfile::NamedTempFile;

// ── Helpers ──────────────────────────────────────────────────────────────────

fn head() -> Command {
    Command::cargo_bin("head").unwrap()
}

/// Write lines to a temp file and return the handle.
fn make_file(content: &[u8]) -> NamedTempFile {
    let mut f = NamedTempFile::new().unwrap();
    f.write_all(content).unwrap();
    f
}

/// Ten newline-separated lines (one through ten) ending with a newline.
fn ten_lines() -> Vec<u8> {
    (1..=10)
        .flat_map(|i| format!("line{i}\n").into_bytes())
        .collect()
}

// ── Default behaviour (first 10 lines) ───────────────────────────────────────

#[test]
fn default_first_10_lines() {
    let content: Vec<u8> = (1..=20)
        .flat_map(|i| format!("line{i}\n").into_bytes())
        .collect();
    let f = make_file(&content);

    let expected: Vec<u8> = (1..=10)
        .flat_map(|i| format!("line{i}\n").into_bytes())
        .collect();

    head().arg(f.path()).assert().success().stdout(expected);
}

#[test]
fn default_fewer_than_10_lines() {
    let f = make_file(b"a\nb\nc\n");
    head().arg(f.path()).assert().success().stdout("a\nb\nc\n");
}

// ── -n / --lines ─────────────────────────────────────────────────────────────

#[test]
fn lines_flag_short() {
    let f = make_file(&ten_lines());
    head()
        .args(["-n", "3", f.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout("line1\nline2\nline3\n");
}

#[test]
fn lines_flag_long() {
    let f = make_file(&ten_lines());
    head()
        .args(["--lines=3", f.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout("line1\nline2\nline3\n");
}

#[test]
fn lines_zero() {
    let f = make_file(b"hello\n");
    head()
        .args(["-n", "0", f.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout("");
}

#[test]
fn lines_all_but_last_negative_value() {
    // `-n -3` → print all but the last 3 lines
    let f = make_file(&ten_lines());
    let expected: Vec<u8> = (1..=7)
        .flat_map(|i| format!("line{i}\n").into_bytes())
        .collect();
    head()
        .args(["-n", "-3", f.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(expected);
}

#[test]
fn lines_all_but_last_long_form() {
    let f = make_file(&ten_lines());
    let expected: Vec<u8> = (1..=8)
        .flat_map(|i| format!("line{i}\n").into_bytes())
        .collect();
    head()
        .args(["--lines=-2", f.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(expected);
}

#[test]
fn lines_all_but_last_exceeds_total() {
    let f = make_file(b"a\nb\nc\n");
    head()
        .args(["-n", "-100", f.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout("");
}

// ── Legacy -NUM shorthand ────────────────────────────────────────────────────

#[test]
fn legacy_shorthand_lines() {
    let f = make_file(&ten_lines());
    head()
        .args(["-3", f.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout("line1\nline2\nline3\n");
}

// ── -c / --bytes ─────────────────────────────────────────────────────────────

#[test]
fn bytes_flag_short() {
    let f = make_file(b"hello world\n");
    head()
        .args(["-c", "5", f.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout("hello");
}

#[test]
fn bytes_flag_long() {
    let f = make_file(b"hello world\n");
    head()
        .args(["--bytes=5", f.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout("hello");
}

#[test]
fn bytes_zero() {
    let f = make_file(b"hello\n");
    head()
        .args(["-c", "0", f.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout("");
}

#[test]
fn bytes_all_but_last() {
    // `-c -3` → print all but the last 3 bytes
    let f = make_file(b"hello\n");
    head()
        .args(["-c", "-3", f.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout("hel");
}

#[test]
fn bytes_all_but_last_long_form() {
    let f = make_file(b"hello\n");
    head()
        .args(["--bytes=-3", f.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout("hel");
}

#[test]
fn bytes_suffix_k() {
    // 1K = 1024 bytes
    let content = vec![b'x'; 2048];
    let f = make_file(&content);
    let result = head()
        .args(["-c", "1K", f.path().to_str().unwrap()])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    assert_eq!(result.len(), 1024);
}

#[test]
fn bytes_suffix_b() {
    // 1b = 512 bytes
    let content = vec![b'y'; 1024];
    let f = make_file(&content);
    let result = head()
        .args(["-c", "1b", f.path().to_str().unwrap()])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    assert_eq!(result.len(), 512);
}

// ── Multiple files with headers ───────────────────────────────────────────────

#[test]
fn multiple_files_headers() {
    let f1 = make_file(b"file1_line1\nfile1_line2\n");
    let f2 = make_file(b"file2_line1\nfile2_line2\n");

    let out = head()
        .args([
            "-n",
            "1",
            f1.path().to_str().unwrap(),
            f2.path().to_str().unwrap(),
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let text = String::from_utf8(out).unwrap();
    assert!(text.contains("==> "));
    assert!(text.contains(" <=="));
    assert!(text.contains("file1_line1"));
    assert!(text.contains("file2_line1"));
}

#[test]
fn single_file_no_header_by_default() {
    let f = make_file(b"hello\n");
    head()
        .args(["-n", "1", f.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("==>").not());
}

// ── -v / --verbose ────────────────────────────────────────────────────────────

#[test]
fn verbose_single_file() {
    let f = make_file(b"hello\n");
    head()
        .args(["-v", "-n", "1", f.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("==>"))
        .stdout(predicate::str::contains("<=="));
}

#[test]
fn verbose_long() {
    let f = make_file(b"hello\n");
    head()
        .args(["--verbose", "-n", "1", f.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("==>"));
}

// ── -q / --quiet / --silent ───────────────────────────────────────────────────

#[test]
fn quiet_suppresses_headers() {
    let f1 = make_file(b"aaa\n");
    let f2 = make_file(b"bbb\n");
    head()
        .args([
            "-q",
            "-n",
            "1",
            f1.path().to_str().unwrap(),
            f2.path().to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("==>").not());
}

#[test]
fn quiet_long() {
    let f1 = make_file(b"aaa\n");
    let f2 = make_file(b"bbb\n");
    head()
        .args([
            "--quiet",
            "-n",
            "1",
            f1.path().to_str().unwrap(),
            f2.path().to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("==>").not());
}

#[test]
fn silent_alias() {
    let f1 = make_file(b"aaa\n");
    let f2 = make_file(b"bbb\n");
    head()
        .args([
            "--silent",
            "-n",
            "1",
            f1.path().to_str().unwrap(),
            f2.path().to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("==>").not());
}

// ── -z / --zero-terminated ───────────────────────────────────────────────────

#[test]
fn zero_terminated_lines() {
    // NUL-delimited records
    let input = b"alpha\0beta\0gamma\0delta\0";
    let f = make_file(input);
    head()
        .args(["-z", "-n", "2", f.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(b"alpha\0beta\0" as &[u8]);
}

#[test]
fn zero_terminated_long() {
    let input = b"alpha\0beta\0gamma\0";
    let f = make_file(input);
    head()
        .args(["--zero-terminated", "-n", "2", f.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(b"alpha\0beta\0" as &[u8]);
}

// ── stdin ─────────────────────────────────────────────────────────────────────

#[test]
fn stdin_default() {
    head()
        .write_stdin("line1\nline2\nline3\n")
        .assert()
        .success()
        .stdout("line1\nline2\nline3\n");
}

#[test]
fn stdin_explicit_dash() {
    head()
        .args(["-n", "2", "-"])
        .write_stdin("a\nb\nc\n")
        .assert()
        .success()
        .stdout("a\nb\n");
}

// ── Error handling ────────────────────────────────────────────────────────────

#[test]
fn nonexistent_file_exits_nonzero() {
    head()
        .arg("/tmp/this_file_does_not_exist_12345")
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot open"));
}

#[test]
fn invalid_lines_count_exits_nonzero() {
    let f = make_file(b"x\n");
    head()
        .args(["-n", "abc", f.path().to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid"));
}

#[test]
fn invalid_bytes_count_exits_nonzero() {
    let f = make_file(b"x\n");
    head()
        .args(["-c", "xyz", f.path().to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid"));
}

// ── No trailing newline edge cases ────────────────────────────────────────────

#[test]
fn no_trailing_newline_last_line_included() {
    // File "a\nb\nc" (no trailing NL) – head -n 3 must include "c"
    let f = make_file(b"a\nb\nc");
    head()
        .args(["-n", "3", f.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout("a\nb\nc");
}

#[test]
fn no_trailing_newline_partial_output() {
    let f = make_file(b"a\nb\nc");
    head()
        .args(["-n", "2", f.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout("a\nb\n");
}

// ── Blank-line separator between headers ─────────────────────────────────────

#[test]
fn blank_line_between_multiple_files() {
    let f1 = make_file(b"hello\n");
    let f2 = make_file(b"world\n");

    let out = head()
        .args([
            "-n",
            "1",
            f1.path().to_str().unwrap(),
            f2.path().to_str().unwrap(),
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let text = String::from_utf8(out).unwrap();
    // The blank line before the second header should appear in the output.
    assert!(
        text.contains("\n\n==>"),
        "expected blank line before second header, got: {text:?}"
    );
}

// ── --help / --version ────────────────────────────────────────────────────────

#[test]
fn help_flag() {
    head().arg("--help").assert().success();
}

#[test]
fn version_flag() {
    head()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}
