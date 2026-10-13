use std::{fmt, str::FromStr};

use chrono::{Datelike, NaiveDate};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct YearMonth {
    year: i32,
    month: u32,
}

impl YearMonth {
    pub fn new(year: i32, month: u32) -> Option<Self> {
        NaiveDate::from_ymd_opt(year, month, 1).map(|_| YearMonth { year, month })
    }

    pub fn of(date: NaiveDate) -> Self {
        YearMonth {
            year: date.year(),
            month: date.month(),
        }
    }

    pub const fn year(self) -> i32 {
        self.year
    }

    pub const fn month(self) -> u32 {
        self.month
    }

    pub fn first_day(self) -> NaiveDate {
        NaiveDate::from_ymd_opt(self.year, self.month, 1)
            .expect("YearMonth always holds a valid month")
    }

    pub fn next(self) -> Self {
        let (year, month) = if self.month == 12 {
            (self.year + 1, 1)
        } else {
            (self.year, self.month + 1)
        };
        Self::new(year, month).expect("YearMonth out of range")
    }

    pub fn prev(self) -> Self {
        let (year, month) = if self.month == 1 {
            (self.year - 1, 12)
        } else {
            (self.year, self.month - 1)
        };
        Self::new(year, month).expect("YearMonth out of range")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseYearMonthError {
    InvalidFormat,
    InvalidMonth,
}

impl fmt::Display for ParseYearMonthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            ParseYearMonthError::InvalidFormat => "month must be in a format YYYY-MM",
            ParseYearMonthError::InvalidMonth => "month must be between 01 and 12",
        };
        f.write_str(message)
    }
}

impl std::error::Error for ParseYearMonthError {}

impl FromStr for YearMonth {
    type Err = ParseYearMonthError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (year, month) = s
            .split_once('-')
            .ok_or(ParseYearMonthError::InvalidFormat)?;

        let all_digits = |part: &str| part.bytes().all(|b| b.is_ascii_digit());
        if year.len() != 4 || month.len() != 2 || !all_digits(year) || !all_digits(month) {
            return Err(ParseYearMonthError::InvalidFormat);
        }

        let year: i32 = year
            .parse()
            .map_err(|_| ParseYearMonthError::InvalidFormat)?;
        let month: u32 = month
            .parse()
            .map_err(|_| ParseYearMonthError::InvalidFormat)?;

        YearMonth::new(year, month).ok_or(ParseYearMonthError::InvalidMonth)
    }
}

impl fmt::Display for YearMonth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}", self.year, self.month)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ym(year: i32, month: u32) -> YearMonth {
        YearMonth::new(year, month).unwrap()
    }

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).unwrap()
    }

    #[test]
    fn rejects_invalid_months() {
        assert_eq!(YearMonth::new(2026, 0), None);
        assert_eq!(YearMonth::new(2026, 13), None);
    }

    #[test]
    fn takes_month_of_a_date() {
        assert_eq!(YearMonth::of(date(2026, 10, 31)), ym(2026, 10));
        assert_eq!(ym(2026, 10).first_day(), date(2026, 10, 1));
    }

    #[test]
    fn steps_across_year_boundaries() {
        assert_eq!(ym(2026, 12).next(), ym(2027, 1));
        assert_eq!(ym(2027, 1).prev(), ym(2026, 12));
        assert_eq!(ym(2026, 6).next().prev(), ym(2026, 6));
    }

    #[test]
    fn order_chronologically() {
        assert!(ym(2025, 12) < ym(2026, 1));
        assert!(ym(2026, 2) < ym(2026, 10));
    }

    #[test]
    fn displays_and_parses() {
        for (text, value) in [("2026-10", ym(2026, 10)), ("0999-01", ym(999, 1))] {
            assert_eq!(value.to_string(), text);
            assert_eq!(text.parse::<YearMonth>(), Ok(value));
        }
    }

    #[test]
    fn rejects_bad_text() {
        use ParseYearMonthError::*;
        let cases = [
            ("", InvalidFormat),
            ("2026", InvalidFormat),
            ("2026-1", InvalidFormat),
            ("26-10", InvalidFormat),
            ("2026/10", InvalidFormat),
            ("2026-10-01", InvalidFormat),
            ("+026-10", InvalidFormat),
            ("2026-00", InvalidMonth),
            ("2026-13", InvalidMonth),
        ];

        for (input, expected) in cases {
            assert_eq!(
                input.parse::<YearMonth>(),
                Err(expected),
                "input: {input:?}"
            );
        }
    }
}
