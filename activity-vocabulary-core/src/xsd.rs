use std::{
    collections::HashMap,
    fmt::{Display, Write},
    str::FromStr,
};

use chrono::{Datelike, FixedOffset, Timelike};
use nom::{
    bytes::complete::tag,
    character::complete::{i64, u64},
    combinator::{eof, opt},
    sequence::tuple,
    IResult,
};
use serde::{Deserialize, Serialize};

pub type LangContainer<T> = HashMap<String, T>;

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum DateTime {
    Naive(chrono::NaiveDateTime),
    WithOffset(chrono::DateTime<FixedOffset>),
}

impl FromStr for DateTime {
    type Err = chrono::ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(with_offset) = chrono::DateTime::<FixedOffset>::parse_from_rfc3339(s) {
            Ok(Self::WithOffset(with_offset))
        } else {
            Ok(Self::Naive(chrono::NaiveDateTime::parse_from_str(
                s,
                "%Y-%m-%dT%H:%M:%S%.f",
            )?))
        }
    }
}

impl<'de> Deserialize<'de> for DateTime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let src: String = <String as Deserialize>::deserialize(deserializer)?;
        Self::from_str(&src).map_err(serde::de::Error::custom)
    }
}

impl Display for DateTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Naive(naive) => {
                let year = naive.year();
                let month = naive.month();
                let day = naive.day();
                let hour = naive.hour();
                let minute = naive.minute();
                let second = naive.second();
                let submilli = naive.timestamp_subsec_millis();
                f.write_fmt(format_args!(
                    "{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}.{submilli:04}"
                ))
            }
            Self::WithOffset(datetime) => {
                f.write_str(&datetime.to_rfc3339_opts(chrono::SecondsFormat::Secs, false))
            }
        }
    }
}

impl Serialize for DateTime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Duration {
    pub negative: bool,
    pub years: u64,
    pub months: u64,
    pub days: u64,
    pub duration: chrono::Duration,
}

impl Display for Duration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char('P')?;
        if self.negative {
            f.write_char('-')?;
        }
        if self.years != 0 {
            f.write_fmt(format_args!("{}Y", self.years))?;
        }
        if self.months != 0 {
            f.write_fmt(format_args!("{}M", self.years))?;
        }
        if self.days != 0 {
            f.write_fmt(format_args!("{}D", self.days))?;
        }
        if !self.duration.num_seconds() != 0 {
            f.write_char('T')?;
            if self.duration.num_hours() != 0 {
                f.write_fmt(format_args!("{}H", self.duration.num_hours()))?;
            }
            if self.duration.num_minutes() % 60 != 0 {
                f.write_fmt(format_args!("{}M", self.duration.num_minutes() % 60))?;
            }
            if self.duration.num_seconds() % 60 != 0 {
                f.write_fmt(format_args!("{}S", self.duration.num_seconds() % 60))?;
            }
        }
        Ok(())
    }
}

fn parse_duration_time_section(src: &str) -> IResult<&str, (i64, i64, i64)> {
    let (src, _) = tag("T")(src)?;
    let (src, hours) = opt(tuple((i64, tag("H"))))(src)?;
    let hours = hours.map(|(n, _)| n).unwrap_or(0);
    let (src, minutes) = opt(tuple((i64, tag("M"))))(src)?;
    let minutes = minutes.map(|(n, _)| n).unwrap_or(0);
    let (src, seconds) = opt(tuple((i64, tag("S"))))(src)?;
    let seconds = seconds.map(|(n, _)| n).unwrap_or(0);
    let (src, _) = eof(src)?;
    Ok((src, (hours, minutes, seconds)))
}

fn parse_duration(src: &str) -> IResult<&str, Duration> {
    let (src, _) = tag("P")(src)?;
    let (src, negative) = opt(tag("-"))(src)?;
    let (src, years) = opt(tuple((u64, tag("Y"))))(src)?;
    let years = years.map(|(n, _)| n).unwrap_or(0);
    let (src, months) = opt(tuple((u64, tag("M"))))(src)?;
    let months = months.map(|(n, _)| n).unwrap_or(0);
    let (src, days) = opt(tuple((u64, tag("D"))))(src)?;
    let days = days.map(|(n, _)| n).unwrap_or(0);
    let (src, (hours, minutes, seconds)) = parse_duration_time_section(src)?;
    let (_, _) = eof(src)?;

    Ok((
        src,
        Duration {
            negative: negative.is_some(),
            years,
            months,
            days,
            duration: chrono::Duration::hours(hours)
                + chrono::Duration::minutes(minutes)
                + chrono::Duration::seconds(seconds),
        },
    ))
}

#[derive(Debug)]
pub struct DurationParseError(String);
impl Display for DurationParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for DurationParseError {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        None
    }

    fn description(&self) -> &str {
        &self.0
    }

    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl FromStr for Duration {
    type Err = DurationParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (_, duration) = parse_duration(s).map_err(|e| DurationParseError(e.to_string()))?;
        Ok(duration)
    }
}

impl Serialize for Duration {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Duration {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = <&str as Deserialize>::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}
