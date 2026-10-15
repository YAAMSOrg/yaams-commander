//! Display formatting for ops-style values. All times are UTC.

use chrono::{DateTime, NaiveDateTime, Utc};

/// `1234567` -> `1,234,567`.
pub fn thousands(value: u64) -> String {
    let digits = value.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

/// Cruise altitude as flight level from 10,000 ft, feet below.
pub fn altitude(feet: u32) -> String {
    if feet >= 10_000 {
        format!("FL{:03}", feet / 100)
    } else {
        format!("{} ft", thousands(feet.into()))
    }
}

/// `2026-07-11 10:00Z`.
pub fn block_time(time: Option<NaiveDateTime>) -> String {
    time.map_or_else(|| "-".into(), |t| t.format("%Y-%m-%d %H:%MZ").to_string())
}

/// `10:00Z`.
pub fn clock(time: Option<NaiveDateTime>) -> String {
    time.map_or_else(|| "-".into(), |t| t.format("%H:%MZ").to_string())
}

/// `11 JUL 26`.
pub fn short_date(time: Option<NaiveDateTime>) -> String {
    time.map_or_else(|| "-".into(), |t| t.format("%d %b %y").to_string().to_uppercase())
}

/// Parses an API timestamp (`2026-07-11T11:35:00.000000Z`) for display.
pub fn timestamp(value: Option<&str>) -> String {
    value.and_then(|v| DateTime::parse_from_rfc3339(v).ok()).map_or_else(
        || "-".into(),
        |t| t.with_timezone(&Utc).format("%Y-%m-%d %H:%MZ").to_string(),
    )
}

/// Minutes as `HH:MM`.
pub fn hhmm(minutes: i64) -> String {
    format!("{:02}:{:02}", minutes / 60, minutes % 60)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_values() {
        assert_eq!(thousands(0), "0");
        assert_eq!(thousands(4200), "4,200");
        assert_eq!(thousands(1_234_567), "1,234,567");
        assert_eq!(altitude(36000), "FL360");
        assert_eq!(altitude(5500), "5,500 ft");
        assert_eq!(hhmm(90), "01:30");
        assert_eq!(timestamp(Some("2026-07-11T11:35:00.000000Z")), "2026-07-11 11:35Z");
        assert_eq!(timestamp(None), "-");
    }
}
