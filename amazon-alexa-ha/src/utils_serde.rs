//! # iso-8601 duration serializing and deserializing (P2D4.2S, PT1210)
//! ISO 8601 Durations are expressed using the following format, where (n) is replaced by the value for each of the date and time elements that follow the (n):
//!
//! P(n)Y(n)M(n)DT(n)H(n)M(n)S
//!
//! Where:
//!
//! * P is the duration designator (referred to as "period"), and is always placed at the beginning of the duration.
//! * Y is the year designator that follows the value for the number of years.
//! * M is the month designator that follows the value for the number of months.
//! * W is the week designator that follows the value for the number of weeks.
//! * D is the day designator that follows the value for the number of days.
//! * T is the time designator that precedes the time components.
//! * H is the hour designator that follows the value for the number of hours.
//! * M is the minute designator that follows the value for the number of minutes.
//! * S is the second designator that follows the value for the number of seconds.
//! For example:
//!
//! P3Y6M4DT12H30M5S
//!
//! Warning:
//! https://developer.amazon.com/de/docs/device-apis/alexa-property-schemas.html#duration defines
//! negative durations. This is a violation of iso-8601 and **utterly nonsense** so not supported.

use std::iter::Peekable;
use std::str::{from_utf8_unchecked, FromStr};
use std::time::Duration;

use chrono::{Datelike, Timelike};
use core::fmt;
use serde::de::{self, Visitor};
use serde::ser::{self, Serialize, Serializer, SerializeSeq};
use serde::{Deserialize, Deserializer};
use std::io::{Cursor, Write};
use std::collections::BTreeMap;

/// Serialize something like ["abc", "def"] to [{"name":"abc"}, {"name":"def"}]
pub struct ArrayOfStaticStrings(pub &'static [&'static str]);

impl Serialize for ArrayOfStaticStrings {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
        for element in self.0 {
            let mut map: BTreeMap<&str, &str> = BTreeMap::new();
            map.insert("name", element);
            seq.serialize_element(&map)?;
        }
        seq.end()
    }
}

/// The deserializer purely exists for tests. Because we cannot deserialize into static strings,
/// this is a no-op.
struct ArrayOfStaticStringsVisitor;

impl<'de> Visitor<'de> for ArrayOfStaticStringsVisitor {
    type Value = ArrayOfStaticStrings;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an integer between -2^31 and 2^31")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
    {
        Ok(ArrayOfStaticStrings(&[]))
    }
}

impl<'de> Deserialize<'de> for ArrayOfStaticStrings {
    fn deserialize<D>(deserializer: D) -> Result<ArrayOfStaticStrings, D::Error>
        where
            D: Deserializer<'de>,
    {
        deserializer.deserialize_str(ArrayOfStaticStringsVisitor)
    }
}


const SECS_PER_MINUTE: u64 = 60;
const SECS_PER_HOUR: u64 = 60 * SECS_PER_MINUTE;
const SECS_PER_DAY: u64 = 24 * SECS_PER_HOUR;
const SECS_PER_WEEK: u64 = 7 * SECS_PER_DAY;

fn to_nanos<S: AsRef<str>>(s: S) -> Option<u32> {
    let s = s.as_ref();

    const NANO_DIGITS: usize = 9;
    if s.len() > NANO_DIGITS {
        return None;
    }

    let extra_zeros = (NANO_DIGITS - s.len()) as u32;
    let mul = 10u32.pow(extra_zeros);
    match u32::from_str(s) {
        Ok(num) => Some(num * mul),
        Err(_) => None,
    }
}

struct Parts<'s> {
    inner: &'s str,
}

impl<'s> Parts<'s> {
    fn new(inner: &str) -> Parts {
        Parts { inner }
    }
}

impl<'s> Iterator for Parts<'s> {
    type Item = (&'s str, char);

    fn next(&mut self) -> Option<(&'s str, char)> {
        self.inner
            .find(|c: char| c.is_ascii_alphabetic())
            .map(|next| {
                let (init, point) = self.inner.split_at(next);
                self.inner = &point[1..];
                (init, point.as_bytes()[0].to_ascii_uppercase() as char)
            })
    }
}

fn maybe_take(
    parts: &mut Peekable<Parts>,
    token: char,
    mul: u64,
) -> Result<u64, std::num::ParseIntError> {
    Ok(match parts.peek().cloned() {
        Some((body, found_token)) if found_token == token => {
            parts.next().unwrap();
            u64::from_str(body)? * mul
        }
        _ => 0,
    })
}

fn take_empty(parts: &mut Peekable<Parts>, token: char) -> Result<(), std::io::Error> {
    match parts.next() {
        Some(("", avail)) if avail == token => Ok(()),
        Some((head, avail)) if avail == token => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("invalid data before '{}': {:?}", token, head),
            ));
        }
        other => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("expected '{}', not {:?}", token, other),
            ));
        }
    }
}

fn parse(input: &str) -> Result<Duration, Box<dyn std::error::Error>> {
    let mut parts = Parts::new(input).peekable();

    let mut seconds = 0u64;
    let mut nanos = 0u32;

    take_empty(&mut parts, 'P')?;

    seconds += maybe_take(&mut parts, 'W', SECS_PER_WEEK)?;
    seconds += maybe_take(&mut parts, 'D', SECS_PER_DAY)?;

    take_empty(&mut parts, 'T')?;

    seconds += maybe_take(&mut parts, 'H', SECS_PER_HOUR)?;
    seconds += maybe_take(&mut parts, 'M', SECS_PER_MINUTE)?;

    if let Some((mut body, 'S')) = parts.peek() {
        parts.next().unwrap();

        if let Some(first_point) = body.find('.') {
            let (main, after) = body.split_at(first_point);
            body = main;
            nanos = to_nanos(&after[1..]).ok_or(std::io::Error::new(
                std::io::ErrorKind::Other,
                "invalid nanos",
            ))?;
        }

        seconds += u64::from_str(body)?;
    }

    if parts.peek().is_some() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("unexpected trailing data: {:?}", parts.next().unwrap()),
        )
            .into());
    }

    Ok(Duration::new(seconds, nanos))
}

pub struct DurationISO8601 {
    duration: std::time::Duration,
}

impl From<std::time::Duration> for DurationISO8601 {
    fn from(duration: Duration) -> Self {
        DurationISO8601 { duration }
    }
}

impl AsRef<std::time::Duration> for DurationISO8601 {
    fn as_ref(&self) -> &Duration {
        &self.duration
    }
}

/// Format is P(n)Y(n)M(n)DT(n)H(n)M(n)S. The string is at maximum 8+14 chars long
fn encode(duration: Duration, buffer: &mut [u8; 32]) -> Result<u64, std::io::Error> {
    let date_time = chrono::NaiveDateTime::from_timestamp_opt(duration.as_secs() as i64, 0)
        .expect("Duration->NaiveDateTime conversion");

    let mut writer = Cursor::new(buffer.as_mut());
    writer.write_all(b"P")?;
    if date_time.year() > 1970 {
        writer.write_all((date_time.year() - 1970).to_string().as_bytes())?;
        writer.write_all(b"Y")?;
    }
    if date_time.month() > 1 {
        writer.write_all((date_time.month() - 1).to_string().as_bytes())?;
        writer.write_all(b"M")?;
    }
    if date_time.day() > 1 {
        writer.write_all((date_time.day() - 1).to_string().as_bytes())?;
        writer.write_all(b"D")?;
    }
    writer.write_all(b"T")?;
    if date_time.hour() > 0 {
        writer.write_all(date_time.hour().to_string().as_bytes())?;
        writer.write_all(b"H")?;
    }
    if date_time.minute() > 0 {
        writer.write_all(date_time.minute().to_string().as_bytes())?;
        writer.write_all(b"M")?;
    }
    if date_time.second() > 0 {
        writer.write_all(date_time.second().to_string().as_bytes())?;
        writer.write_all(b"S")?;
    }
    if duration.subsec_nanos() > 0 {
        writer.write_all(b".")?;
        writer.write_all(duration.subsec_nanos().to_string().as_bytes())?;
    }
    Ok(writer.position())
}

impl Serialize for DurationISO8601 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        let mut buffer: [u8; 32] = [0; 32];
        let len =
            encode(self.duration, &mut buffer).map_err(|io| ser::Error::custom(io.to_string()))?;

        serializer.serialize_str(unsafe { from_utf8_unchecked(&buffer[0..len as usize]) })
    }
}

struct DurationISO8601Visitor;

impl<'de> Visitor<'de> for DurationISO8601Visitor {
    type Value = DurationISO8601;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an integer between -2^31 and 2^31")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
    {
        Ok(DurationISO8601 {
            duration: parse(v).map_err(|e| de::Error::custom(e.to_string()))?,
        })
    }
}

impl<'de> Deserialize<'de> for DurationISO8601 {
    fn deserialize<D>(deserializer: D) -> Result<DurationISO8601, D::Error>
        where
            D: Deserializer<'de>,
    {
        deserializer.deserialize_str(DurationISO8601Visitor)
    }
}

#[cfg(test)]
mod tests {
    use super::DurationISO8601;
    use std::time::Duration;

    #[test]
    fn test_serialize() {
        let t = DurationISO8601 {
            duration: Duration::new(10, 100),
        };
        let str = serde_json::to_string(&t).unwrap();
        assert_eq!(str, "\"PT10S.100\"");

        let t = DurationISO8601 {
            duration: Duration::new(10, 0),
        };
        let str = serde_json::to_string(&t).unwrap();
        assert_eq!(str, "\"PT10S\"");

        let t = DurationISO8601 {
            duration: Duration::new(60 + 10, 0),
        };
        let str = serde_json::to_string(&t).unwrap();
        assert_eq!(str, "\"PT1M10S\"");

        let t: DurationISO8601 = serde_json::from_str(&str).unwrap();
        assert_eq!(70, t.as_ref().as_secs());
    }

    #[test]
    fn test_nanos() {
        use super::to_nanos;
        assert_eq!(0, to_nanos("0").unwrap());
        assert_eq!(0, to_nanos("000").unwrap());

        assert_eq!(1, to_nanos("000000001").unwrap());
        assert_eq!(10, to_nanos("00000001").unwrap());
        assert_eq!(100, to_nanos("0000001").unwrap());
        assert_eq!(1000, to_nanos("000001").unwrap());
        assert_eq!(10000, to_nanos("00001").unwrap());
        assert_eq!(100000, to_nanos("0001").unwrap());
        assert_eq!(1000000, to_nanos("001").unwrap());
        assert_eq!(10000000, to_nanos("01").unwrap());
        assert_eq!(100000000, to_nanos("1").unwrap());

        assert_eq!(7_010, to_nanos("00000701").unwrap());
    }

    #[test]
    fn duration() {
        use super::parse;
        assert_eq!(Duration::new(7, 0), parse("PT7S").unwrap());
        assert_eq!(Duration::new(7, 5_000_000), parse("PT7.005S").unwrap());
        assert_eq!(Duration::new(2 * 60, 0), parse("PT2M").unwrap());
        assert_eq!(
            Duration::new((2 * 24 + 1) * 60 * 60, 0),
            parse("P2DT1H").unwrap()
        );
    }

    #[test]
    fn parts() {
        let mut p = super::Parts::new("1D23M");
        assert_eq!(Some(("1", 'D')), p.next());
        assert_eq!(Some(("23", 'M')), p.next());
    }
}
