use std::time::Duration;

pub fn parse_interval(interval: &str) -> Result<Duration, String> {
    let last = interval.chars().last().unwrap_or('\0');

    if !"smh".contains(last) {
        return Err(format!(
            "Invalid interval format: '{}'. \
             Use a number followed by s, m, or h (e.g., 30s, 10m, 2h).",
            interval
        ));
    }

    let num_part = &interval[..interval.len() - 1];
    let n = num_part.parse::<u64>().map_err(|_| {
        format!(
            "Invalid number in '{}'. Use a positive integer (e.g., 30s, 10m, 2h).",
            interval
        )
    })?;

    if n == 0 {
        return Err("The interval must be greater than zero.".into());
    }

    match last {
        's' => Ok(Duration::from_secs(n)),
        'm' => Ok(Duration::from_secs(n * 60)),
        'h' => Ok(Duration::from_secs(n * 3600)),
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::parse_interval;

    #[test]
    fn parse_valid_second_interval() {
        assert_eq!(parse_interval("30s").expect("valid seconds"), Duration::from_secs(30));
    }

    #[test]
    fn parse_valid_minute_interval() {
        assert_eq!(parse_interval("10m").expect("valid minutes"), Duration::from_secs(600));
    }

    #[test]
    fn parse_valid_hour_interval() {
        assert_eq!(parse_interval("2h").expect("valid hours"), Duration::from_secs(7200));
    }

    #[test]
    fn reject_invalid_suffix() {
        let err = parse_interval("10d").expect_err("invalid suffix should fail");
        assert!(err.contains("Invalid interval format"), "unexpected error: {err}");
    }

    #[test]
    fn reject_non_numeric_prefix() {
        let err = parse_interval("xxm").expect_err("non-numeric interval should fail");
        assert!(err.contains("Invalid number"), "unexpected error: {err}");
    }

    #[test]
    fn reject_zero_value() {
        let err = parse_interval("0s").expect_err("zero interval should fail");
        assert_eq!(err, "The interval must be greater than zero.");
    }
}