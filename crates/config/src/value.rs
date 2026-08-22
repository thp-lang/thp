use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// A finite number of bytes.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ByteSize(u64);

impl ByteSize {
    pub const fn from_bytes(bytes: u64) -> Self {
        Self(bytes)
    }

    pub const fn bytes(self) -> u64 {
        self.0
    }
}

/// A finite whole number of seconds.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Duration(u64);

impl Duration {
    pub const fn from_seconds(seconds: u64) -> Self {
        Self(seconds)
    }

    pub const fn seconds(self) -> u64 {
        self.0
    }
}

/// A resource limit that can be finite or unlimited.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Limit<T> {
    Unlimited,
    Finite(T),
}

impl<T> Limit<T> {
    pub const fn finite(self) -> Option<T>
    where
        T: Copy,
    {
        match self {
            Self::Unlimited => None,
            Self::Finite(value) => Some(value),
        }
    }

    pub const fn is_unlimited(&self) -> bool {
        matches!(self, Self::Unlimited)
    }
}

/// Why a textual resource limit was rejected.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
#[error("{message}")]
pub struct ParseLimitError {
    message: String,
}

impl ParseLimitError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl FromStr for Limit<ByteSize> {
    type Err = ParseLimitError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        parse_limit(input, SizeUnit::parse).map(|value| {
            value.map_or(Self::Unlimited, |value| {
                Self::Finite(ByteSize::from_bytes(value))
            })
        })
    }
}

impl FromStr for Limit<Duration> {
    type Err = ParseLimitError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        parse_limit(input, TimeUnit::parse).map(|value| {
            value.map_or(Self::Unlimited, |value| {
                Self::Finite(Duration::from_seconds(value))
            })
        })
    }
}

fn parse_limit(
    input: &str,
    unit: impl FnOnce(&str) -> Result<u64, ParseLimitError>,
) -> Result<Option<u64>, ParseLimitError> {
    if input == "unlimited" {
        return Ok(None);
    }
    if input.is_empty() || input.bytes().any(|byte| byte.is_ascii_whitespace()) {
        return Err(ParseLimitError::new(
            "must be an integer and unit with no whitespace",
        ));
    }
    let digit_count = input.bytes().take_while(u8::is_ascii_digit).count();
    if digit_count == 0 {
        return Err(ParseLimitError::new(
            "must begin with a non-negative integer",
        ));
    }
    let (number, suffix) = input.split_at(digit_count);
    let number = number
        .parse::<u64>()
        .map_err(|_| ParseLimitError::new("integer is too large"))?;
    let multiplier = unit(suffix)?;
    let value = number
        .checked_mul(multiplier)
        .ok_or_else(|| ParseLimitError::new("value overflows a 64-bit unsigned integer"))?;
    Ok((value != 0).then_some(value))
}

struct SizeUnit;

impl SizeUnit {
    fn parse(suffix: &str) -> Result<u64, ParseLimitError> {
        match suffix {
            "" => Ok(1),
            suffix if suffix.eq_ignore_ascii_case("k") => Ok(1 << 10),
            suffix if suffix.eq_ignore_ascii_case("m") => Ok(1 << 20),
            suffix if suffix.eq_ignore_ascii_case("g") => Ok(1 << 30),
            _ => Err(ParseLimitError::new(
                "unsupported size suffix; expected K, M, G, or no suffix",
            )),
        }
    }
}

struct TimeUnit;

impl TimeUnit {
    fn parse(suffix: &str) -> Result<u64, ParseLimitError> {
        match suffix {
            "s" => Ok(1),
            "m" => Ok(60),
            "h" => Ok(60 * 60),
            _ => Err(ParseLimitError::new(
                "unsupported duration suffix; expected s, m, or h",
            )),
        }
    }
}

impl fmt::Display for Limit<ByteSize> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unlimited => formatter.write_str("unlimited"),
            Self::Finite(size) => size.bytes().fmt(formatter),
        }
    }
}

impl fmt::Display for Limit<Duration> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unlimited => formatter.write_str("unlimited"),
            Self::Finite(duration) => duration.seconds().fmt(formatter),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_size_units_and_aliases() {
        assert_eq!(
            "200M".parse(),
            Ok(Limit::Finite(ByteSize::from_bytes(200 * 1024 * 1024)))
        );
        assert_eq!(
            "2g".parse(),
            Ok(Limit::Finite(ByteSize::from_bytes(2 * 1024 * 1024 * 1024)))
        );
        assert_eq!("0M".parse::<Limit<ByteSize>>(), Ok(Limit::Unlimited));
        assert_eq!("unlimited".parse::<Limit<ByteSize>>(), Ok(Limit::Unlimited));
    }

    #[test]
    fn parses_duration_units_and_aliases() {
        assert_eq!(
            "2h".parse(),
            Ok(Limit::Finite(Duration::from_seconds(7200)))
        );
        assert_eq!("0s".parse::<Limit<Duration>>(), Ok(Limit::Unlimited));
    }

    #[test]
    fn rejects_invalid_limits() {
        for value in ["1.5M", "1h30m", " 1M", "1 M", "-1M", "1T", "M"] {
            assert!(value.parse::<Limit<ByteSize>>().is_err(), "{value}");
        }
        for value in ["1.5s", "1ms", "1S", "60", "-1s", "1 s"] {
            assert!(value.parse::<Limit<Duration>>().is_err(), "{value}");
        }
    }

    #[test]
    fn rejects_overflow() {
        assert!("18446744073709551615G".parse::<Limit<ByteSize>>().is_err());
        assert!("18446744073709551615h".parse::<Limit<Duration>>().is_err());
    }

    #[test]
    fn accepts_the_finite_u64_boundary() {
        assert_eq!(
            "18446744073709551615".parse::<Limit<ByteSize>>(),
            Ok(Limit::Finite(ByteSize::from_bytes(u64::MAX)))
        );
        assert_eq!(
            "18446744073709551615s".parse::<Limit<Duration>>(),
            Ok(Limit::Finite(Duration::from_seconds(u64::MAX)))
        );
    }
}
