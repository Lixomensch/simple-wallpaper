use std::path::PathBuf;
use std::thread;
use std::time::Duration;

use colored::Colorize;

use crate::core::{lists, operations};
use crate::error::{IntervalError, SwpError};

pub enum PlaybackSource {
    List(lists::ListPlayer),
    Filesystem,
}

impl PlaybackSource {
    fn apply_next(&mut self) -> Result<PathBuf, SwpError> {
        match self {
            Self::List(player) => {
                let path = player.next_wallpaper()?;
                operations::apply_wallpaper(&path)
            }
            Self::Filesystem => operations::random_wallpaper(),
        }
    }
}

pub fn parse_interval(interval: &str) -> Result<Duration, SwpError> {
    let last = interval.chars().last().unwrap_or('\0');

    if !"smh".contains(last) {
        return Err(IntervalError::InvalidFormat {
            input: interval.to_string(),
        }
        .into());
    }

    let num_part = &interval[..interval.len() - 1];
    let n = num_part
        .parse::<u64>()
        .map_err(|_| IntervalError::InvalidNumber {
            input: interval.to_string(),
        })?;

    if n == 0 {
        return Err(IntervalError::ZeroInterval.into());
    }

    match last {
        's' => Ok(Duration::from_secs(n)),
        'm' => Ok(Duration::from_secs(n * 60)),
        'h' => Ok(Duration::from_secs(n * 3600)),
        _ => unreachable!(),
    }
}

pub fn run(interval: &str, mut source: PlaybackSource) -> Result<(), SwpError> {
    let duration = parse_interval(interval)?;

    loop {
        match source.apply_next() {
            Ok(applied) => println!(
                "  {}",
                applied.file_name().and_then(|n| n.to_str()).unwrap_or("?")
            ),
            Err(e) => eprintln!("  {} {}", "Error:".yellow().bold(), e),
        }
        thread::sleep(duration);
    }
}

pub fn run_foreground(interval: &str, source: PlaybackSource) -> Result<(), SwpError> {
    run(interval, source)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::parse_interval;
    use crate::error::{IntervalError, SwpError};

    #[test]
    fn parse_valid_second_interval() {
        assert_eq!(
            parse_interval("30s").expect("valid seconds"),
            Duration::from_secs(30)
        );
    }

    #[test]
    fn parse_valid_minute_interval() {
        assert_eq!(
            parse_interval("10m").expect("valid minutes"),
            Duration::from_secs(600)
        );
    }

    #[test]
    fn parse_valid_hour_interval() {
        assert_eq!(
            parse_interval("2h").expect("valid hours"),
            Duration::from_secs(7200)
        );
    }

    #[test]
    fn reject_invalid_suffix() {
        let err = parse_interval("10d").expect_err("invalid suffix should fail");
        match err {
            SwpError::Interval(IntervalError::InvalidFormat { input }) => {
                assert_eq!(input, "10d")
            }
            other => panic!("unexpected error variant: {other}"),
        }
    }

    #[test]
    fn reject_non_numeric_prefix() {
        let err = parse_interval("xxm").expect_err("non-numeric interval should fail");
        match err {
            SwpError::Interval(IntervalError::InvalidNumber { input }) => {
                assert_eq!(input, "xxm")
            }
            other => panic!("unexpected error variant: {other}"),
        }
    }

    #[test]
    fn reject_zero_value() {
        let err = parse_interval("0s").expect_err("zero interval should fail");
        match err {
            SwpError::Interval(IntervalError::ZeroInterval) => {}
            other => panic!("unexpected error variant: {other}"),
        }
    }
}
