/// Parse a size string (digits optionally followed by a unit suffix) into a
/// number of bytes or lines.
///
/// Supported suffixes match GNU coreutils:
///
/// | Suffix         | Multiplier             |
/// |----------------|------------------------|
/// | (none)         | 1                      |
/// | `b`            | 512                    |
/// | `kB`           | 1,000                  |
/// | `K`, `KiB`, `k`| 1,024                 |
/// | `MB`           | 1,000,000              |
/// | `M`, `MiB`     | 1,048,576              |
/// | `GB`           | 1,000,000,000          |
/// | `G`, `GiB`     | 1,073,741,824          |
/// | `TB`           | 1,000,000,000,000      |
/// | `T`, `TiB`     | 1,099,511,627,776      |
/// | `PB`           | 10^15                  |
/// | `P`, `PiB`     | 2^50                   |
/// | `EB`           | 10^18                  |
/// | `E`, `EiB`     | 2^60                   |
/// | `ZB`           | 10^21                  |
/// | `Z`, `ZiB`     | 2^70                   |
/// | `YB`           | 10^24                  |
/// | `Y`, `YiB`     | 2^80                   |
/// | `RB`           | 10^27                  |
/// | `R`, `RiB`     | 2^90                   |
/// | `QB`           | 10^30                  |
/// | `Q`, `QiB`     | 2^100                  |
///
/// Returns an error string if the input is invalid or the result overflows
/// `u64`.
pub fn parse_size(s: &str) -> Result<u64, String> {
    if s.is_empty() {
        return Err(format!("invalid count: {s:?}"));
    }

    let (num_part, suffix) = split_num_suffix(s);

    if num_part.is_empty() {
        return Err(format!("invalid count: {s:?}"));
    }

    let base: u128 = num_part
        .parse()
        .map_err(|_| format!("invalid count: {s:?}"))?;

    let multiplier =
        get_multiplier(suffix).ok_or_else(|| format!("invalid suffix in count: {suffix:?}"))?;

    let result = base
        .checked_mul(multiplier)
        .ok_or_else(|| format!("count is too large: {s:?}"))?;

    u64::try_from(result).map_err(|_| format!("count is too large: {s:?}"))
}

/// Split the leading digit sequence from the trailing suffix.
fn split_num_suffix(s: &str) -> (&str, &str) {
    let pos = s.find(|c: char| !c.is_ascii_digit()).unwrap_or(s.len());
    (&s[..pos], &s[pos..])
}

/// Return the numeric multiplier for a unit suffix, or `None` for unknown
/// suffixes.  Uses `u128` internally to represent values larger than `u64`.
fn get_multiplier(suffix: &str) -> Option<u128> {
    Some(match suffix {
        "" => 1,
        "b" => 512,
        "kB" => 1_000,
        // k is not a standard SI prefix but GNU coreutils accepts it
        "k" | "K" | "KiB" => 1_024,
        "MB" => 1_000_000,
        "M" | "MiB" => 1_048_576,
        "GB" => 1_000_000_000,
        "G" | "GiB" => 1_073_741_824,
        "TB" => 1_000_000_000_000,
        "T" | "TiB" => 1_099_511_627_776,
        "PB" => 1_000_000_000_000_000,
        "P" | "PiB" => 1_125_899_906_842_624,
        "EB" => 1_000_000_000_000_000_000,
        "E" | "EiB" => 1_152_921_504_606_846_976,
        "ZB" => 1_000_000_000_000_000_000_000,
        "Z" | "ZiB" => 1_180_591_620_717_411_303_424,
        "YB" => 1_000_000_000_000_000_000_000_000,
        "Y" | "YiB" => 1_208_925_819_614_629_174_706_176,
        "RB" => 1_000_000_000_000_000_000_000_000_000,
        "R" | "RiB" => 1_237_940_039_285_380_274_899_124_224,
        "QB" => 1_000_000_000_000_000_000_000_000_000_000,
        "Q" | "QiB" => 1_267_650_600_228_229_401_496_703_205_376,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plain_numbers() {
        assert_eq!(parse_size("0"), Ok(0));
        assert_eq!(parse_size("1"), Ok(1));
        assert_eq!(parse_size("42"), Ok(42));
        assert_eq!(parse_size("1000"), Ok(1000));
    }

    #[test]
    fn test_block_suffix() {
        assert_eq!(parse_size("2b"), Ok(1024));
        assert_eq!(parse_size("1b"), Ok(512));
    }

    #[test]
    fn test_si_suffixes() {
        assert_eq!(parse_size("1kB"), Ok(1_000));
        assert_eq!(parse_size("1MB"), Ok(1_000_000));
        assert_eq!(parse_size("1GB"), Ok(1_000_000_000));
        assert_eq!(parse_size("1TB"), Ok(1_000_000_000_000));
        assert_eq!(parse_size("1PB"), Ok(1_000_000_000_000_000));
        assert_eq!(parse_size("1EB"), Ok(1_000_000_000_000_000_000));
    }

    #[test]
    fn test_binary_suffixes() {
        assert_eq!(parse_size("1K"), Ok(1_024));
        assert_eq!(parse_size("1KiB"), Ok(1_024));
        assert_eq!(parse_size("1k"), Ok(1_024));
        assert_eq!(parse_size("1M"), Ok(1_048_576));
        assert_eq!(parse_size("1MiB"), Ok(1_048_576));
        assert_eq!(parse_size("1G"), Ok(1_073_741_824));
        assert_eq!(parse_size("1GiB"), Ok(1_073_741_824));
        assert_eq!(parse_size("2K"), Ok(2_048));
    }

    #[test]
    fn test_invalid_suffix() {
        assert!(parse_size("1X").is_err());
        assert!(parse_size("1xyz").is_err());
    }

    #[test]
    fn test_invalid_number() {
        assert!(parse_size("").is_err());
        assert!(parse_size("abc").is_err());
        assert!(parse_size("K").is_err());
    }

    #[test]
    fn test_overflow() {
        // 2 * 10^30 overflows u64
        assert!(parse_size("2QB").is_err());
    }
}
