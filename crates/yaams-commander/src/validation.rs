//! Client-side validation mirroring the server rules for filing a PIREP.
//! The server stays authoritative; these checks give immediate feedback.

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

pub const MAX_BLOCK_MINUTES: i64 = 26 * 60;
pub const MAX_FUEL: u32 = 600_000;

pub type Check<T> = Result<T, String>;

/// 4-letter ICAO airport code, returned uppercased.
pub fn icao(input: &str) -> Check<String> {
    let value = input.trim().to_uppercase();
    if value.is_empty() {
        return Err("Required".into());
    }
    if value.len() != 4 || !value.chars().all(|c| c.is_ascii_alphabetic()) {
        return Err("Must be a 4-letter ICAO code".into());
    }
    Ok(value)
}

/// Optional ICAO filter value (empty = no filter).
pub fn optional_icao(input: &str) -> Check<Option<String>> {
    if input.trim().is_empty() {
        Ok(None)
    } else {
        icao(input).map(Some)
    }
}

/// Numeric flight number with 1-4 digits.
pub fn flight_number(input: &str) -> Check<u32> {
    let value = input.trim();
    if value.is_empty() {
        return Err("Required".into());
    }
    if value.len() > 4 || !value.chars().all(|c| c.is_ascii_digit()) {
        return Err("1-4 digits".into());
    }
    value.parse().map_err(|_| "1-4 digits".into())
}

/// Radio callsign: 1-4 digits optionally followed by up to 2 letters.
pub fn callsign(input: &str) -> Check<String> {
    let value = input.trim().to_uppercase();
    if value.is_empty() {
        return Err("Required".into());
    }
    let digits = value.chars().take_while(char::is_ascii_digit).count();
    let letters = &value[digits..];
    if !(1..=4).contains(&digits) || letters.len() > 2 || !letters.chars().all(|c| c.is_ascii_uppercase()) {
        return Err("1-4 digits, optionally followed by up to 2 letters".into());
    }
    Ok(value)
}

/// Cruise altitude in feet: 1000-60000 in 500 ft steps. Accepts `36000`,
/// `36,000` or flight levels like `FL360`.
pub fn cruise_altitude(input: &str) -> Check<u32> {
    let value = input.trim().to_uppercase().replace([',', '.', ' '], "");
    if value.is_empty() {
        return Err("Required".into());
    }
    let feet = if let Some(level) = value.strip_prefix("FL") {
        level
            .parse::<u32>()
            .map_err(|_| "Use feet (36000) or a flight level (FL360)".to_string())?
            * 100
    } else {
        value
            .parse::<u32>()
            .map_err(|_| "Use feet (36000) or a flight level (FL360)".to_string())?
    };
    if !(1000..=60000).contains(&feet) {
        return Err("Between 1,000 and 60,000 ft".into());
    }
    if feet % 500 != 0 {
        return Err("Must be a multiple of 500 ft".into());
    }
    Ok(feet)
}

/// UTC date `YYYY-MM-DD`.
pub fn date(input: &str) -> Check<NaiveDate> {
    let value = input.trim();
    if value.is_empty() {
        return Err("Required".into());
    }
    NaiveDate::parse_from_str(value, "%Y-%m-%d").map_err(|_| "Use YYYY-MM-DD".into())
}

/// Optional date filter value.
pub fn optional_date(input: &str) -> Check<Option<NaiveDate>> {
    if input.trim().is_empty() {
        Ok(None)
    } else {
        date(input).map(Some)
    }
}

/// UTC time `HH:MM`, `HHMM` or `HH:MM:SS`.
pub fn time(input: &str) -> Check<NaiveTime> {
    let value = input.trim();
    if value.is_empty() {
        return Err("Required".into());
    }
    ["%H:%M", "%H%M", "%H:%M:%S"]
        .iter()
        .find_map(|fmt| NaiveTime::parse_from_str(value, fmt).ok())
        .ok_or_else(|| "Use HH:MM (UTC)".into())
}

/// Block on must be after block off and the flight at most 26 hours.
/// Returns the block time in minutes.
pub fn block_times(off: NaiveDateTime, on: NaiveDateTime) -> Check<i64> {
    let minutes = (on - off).num_minutes();
    if on <= off {
        return Err("Block on must be after block off".into());
    }
    if minutes > MAX_BLOCK_MINUTES {
        return Err("A flight may not exceed 26 hours".into());
    }
    Ok(minutes)
}

/// Burned fuel, 1-600,000 in the airline's unit.
pub fn fuel(input: &str) -> Check<u32> {
    let value = input.trim().replace([',', '.', ' '], "");
    if value.is_empty() {
        return Err("Required".into());
    }
    let amount: u32 = value.parse().map_err(|_| "Whole number".to_string())?;
    if !(1..=MAX_FUEL).contains(&amount) {
        return Err("Between 1 and 600,000".into());
    }
    Ok(amount)
}

/// Filed route string, normalized to single-spaced uppercase.
pub fn route(input: &str) -> Check<String> {
    let value = input.split_whitespace().collect::<Vec<_>>().join(" ").to_uppercase();
    if value.is_empty() {
        return Err("Required (use DCT for direct)".into());
    }
    Ok(value)
}

/// Optional remarks: letters, digits, spaces, `.` and `-` only.
pub fn remarks(input: &str) -> Check<Option<String>> {
    let value = input.trim();
    if value.is_empty() {
        return Ok(None);
    }
    if !value
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, ' ' | '.' | '-'))
    {
        return Err("Only letters, digits, spaces, '.' and '-'".into());
    }
    Ok(Some(value.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icao_codes() {
        assert_eq!(icao(" eddf ").unwrap(), "EDDF");
        assert!(icao("EDD").is_err());
        assert!(icao("ED1F").is_err());
        assert_eq!(optional_icao("").unwrap(), None);
    }

    #[test]
    fn flight_numbers_and_callsigns() {
        assert_eq!(flight_number("421").unwrap(), 421);
        assert!(flight_number("12345").is_err());
        assert!(flight_number("42A").is_err());
        assert_eq!(callsign("421ab").unwrap(), "421AB");
        assert_eq!(callsign("7").unwrap(), "7");
        assert!(callsign("AB").is_err());
        assert!(callsign("12345").is_err());
        assert!(callsign("12ABC").is_err());
    }

    #[test]
    fn cruise_altitudes() {
        assert_eq!(cruise_altitude("36000").unwrap(), 36000);
        assert_eq!(cruise_altitude("FL360").unwrap(), 36000);
        assert_eq!(cruise_altitude("5,500").unwrap(), 5500);
        assert!(cruise_altitude("36100").is_err());
        assert!(cruise_altitude("500").is_err());
        assert!(cruise_altitude("60500").is_err());
    }

    #[test]
    fn times_and_durations() {
        let d = date("2026-07-11").unwrap();
        let off = d.and_time(time("10:00").unwrap());
        let on = d.and_time(time("1130").unwrap());
        assert_eq!(block_times(off, on).unwrap(), 90);
        assert!(block_times(on, off).is_err());
        let long = off + chrono::Duration::minutes(MAX_BLOCK_MINUTES + 1);
        assert!(block_times(off, long).is_err());
        assert!(date("11.07.2026").is_err());
        assert!(time("25:00").is_err());
    }

    #[test]
    fn fuel_route_remarks() {
        assert_eq!(fuel("4,200").unwrap(), 4200);
        assert!(fuel("0").is_err());
        assert!(fuel("600001").is_err());
        assert_eq!(route(" sovat  ul610 konan ").unwrap(), "SOVAT UL610 KONAN");
        assert!(route("  ").is_err());
        assert_eq!(remarks("Smooth flight.").unwrap().as_deref(), Some("Smooth flight."));
        assert!(remarks("Great!").is_err());
        assert_eq!(remarks(" ").unwrap(), None);
    }
}
