use std::{
    fmt,
    iter::Sum,
    ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign},
    str,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, sqlx::Type)]
#[sqlx(transparent)]
pub struct Money(i64);

impl Money {
    pub const ZERO: Money = Money(0);

    pub const fn from_cents(cents: i64) -> Self {
        Money(cents)
    }

    pub const fn cents(self) -> i64 {
        self.0
    }

    pub fn split_evenly(self, parts: u32) -> Vec<Money> {
        assert!(parts > 0, "cannot split money into zero parts");

        let n = i64::from(parts);
        let base = self.0 / n;
        let remainder = self.0 % n;
        let extra = remainder.signum();
        let leftover = remainder.unsigned_abs();

        (0..parts)
            .map(|i| {
                if u64::from(i) < leftover {
                    Money(base + extra)
                } else {
                    Money(base)
                }
            })
            .collect()
    }
}

impl Add for Money {
    type Output = Money;

    fn add(self, rhs: Money) -> Money {
        Money(self.0 + rhs.0)
    }
}

impl Sub for Money {
    type Output = Money;

    fn sub(self, rhs: Money) -> Money {
        Money(self.0 - rhs.0)
    }
}

impl Neg for Money {
    type Output = Money;

    fn neg(self) -> Money {
        Money(-self.0)
    }
}

impl AddAssign for Money {
    fn add_assign(&mut self, rhs: Money) {
        self.0 += rhs.0;
    }
}

impl SubAssign for Money {
    fn sub_assign(&mut self, rhs: Money) {
        self.0 -= rhs.0;
    }
}

impl Mul<i64> for Money {
    type Output = Money;

    fn mul(self, rhs: i64) -> Money {
        Money(self.0 * rhs)
    }
}

impl Sum for Money {
    fn sum<I: Iterator<Item = Money>>(iter: I) -> Money {
        iter.fold(Money::ZERO, Add::add)
    }
}

impl<'a> Sum<&'a Money> for Money {
    fn sum<I: Iterator<Item = &'a Money>>(iter: I) -> Money {
        iter.copied().sum()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseMoneyError {
    Empty,
    InvalidFormat,
    TooManyDecimals,
    OutOfRange,
}

impl fmt::Display for ParseMoneyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            ParseMoneyError::Empty => "amount is empty",
            ParseMoneyError::InvalidFormat => "amount must be in format 1234.56 or -1234.56",
            ParseMoneyError::TooManyDecimals => "amount has more than two decimal places",
            ParseMoneyError::OutOfRange => "amount is too large",
        };
        f.write_str(message)
    }
}

impl std::error::Error for ParseMoneyError {}

impl str::FromStr for Money {
    type Err = ParseMoneyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(ParseMoneyError::Empty);
        }

        let (negative, unsigned) = match s.strip_prefix('-') {
            Some(rest) => (true, rest),
            None => (false, s),
        };

        let (whole, fraction) = match unsigned.split_once('.') {
            Some((_, "")) => return Err(ParseMoneyError::InvalidFormat),
            Some(parts) => parts,
            None => (unsigned, ""),
        };

        let all_digits = |part: &str| part.bytes().all(|b| b.is_ascii_digit());
        if whole.is_empty() || !all_digits(whole) || !all_digits(fraction) {
            return Err(ParseMoneyError::InvalidFormat);
        }

        if fraction.len() > 2 {
            return Err(ParseMoneyError::TooManyDecimals);
        }

        let whole: i64 = whole.parse().map_err(|_| ParseMoneyError::OutOfRange)?;
        let fraction: i64 = format!("{fraction:0<2}")
            .parse()
            .map_err(|_| ParseMoneyError::InvalidFormat)?;

        let cents = whole
            .checked_mul(100)
            .and_then(|c| c.checked_add(fraction))
            .ok_or(ParseMoneyError::OutOfRange)?;

        Ok(Money(if negative { -cents } else { cents }))
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sign = if self.0 < 0 { "-" } else { "" };
        let abs = self.0.unsigned_abs();
        let dollars = group_thousands(abs / 100);
        let cents = abs % 100;
        write!(f, "{sign}${dollars}.{cents:02}")
    }
}

fn group_thousands(n: u64) -> String {
    let digits = n.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (i, ch) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',')
        }
        out.push(ch);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adds_and_subtracts() {
        let a = Money::from_cents(1050);
        let b = Money::from_cents(325);
        assert_eq!(a + b, Money::from_cents(1375));
        assert_eq!(a - b, Money::from_cents(725));
    }

    #[test]
    fn add_and_sub_adding() {
        let mut m = Money::from_cents(100);
        m += Money::from_cents(50);
        m -= Money::from_cents(20);
        assert_eq!(m, Money::from_cents(130));
    }

    #[test]
    fn negates() {
        assert_eq!(-Money::from_cents(500), Money::from_cents(-500));
    }

    #[test]
    fn multiplies_by_integer() {
        assert_eq!(Money::from_cents(250) * 12, Money::from_cents(3000));
    }

    #[test]
    fn sums_owned_and_borrowed() {
        let amounts = vec![
            Money::from_cents(100),
            Money::from_cents(-30),
            Money::from_cents(5),
        ];
        let borrowed: Money = amounts.iter().sum();
        let owned: Money = amounts.into_iter().sum();
        assert_eq!(borrowed, Money::from_cents(75));
        assert_eq!(owned, Money::from_cents(75));
    }

    #[test]
    fn empty_sum_is_zero() {
        let none: Vec<Money> = Vec::new();
        assert_eq!(none.into_iter().sum::<Money>(), Money::ZERO);
    }

    #[test]
    fn orders_by_amount() {
        assert!(Money::from_cents(-1) < Money::ZERO);
        assert!(Money::from_cents(1) > Money::ZERO);
    }

    #[test]
    #[should_panic]
    fn overflow_panics() {
        let _ = Money::from_cents(i64::MAX) + Money::from_cents(1);
    }

    #[test]
    fn parses_valid_amounts() {
        let cases = [
            ("0", 0),
            ("12", 1200),
            ("12.5", 1250),
            ("12.05", 1205),
            ("007.10", 710),
            ("1234.56", 123456),
            ("-1234.56", -123456),
            ("-0.01", -1),
            ("92233720368547758.07", i64::MAX),
        ];

        for (input, cents) in cases {
            assert_eq!(
                input.parse::<Money>(),
                Ok(Money::from_cents(cents)),
                "input: {input:?}"
            );
        }
    }

    #[test]
    fn rejects_invalid_amounts() {
        use ParseMoneyError::*;
        let cases = [
            ("", Empty),
            ("-", InvalidFormat),
            ("abc", InvalidFormat),
            ("12.", InvalidFormat),
            (".50", InvalidFormat),
            ("+12.00", InvalidFormat),
            ("--5", InvalidFormat),
            ("1.-5", InvalidFormat),
            ("1.2.3", InvalidFormat),
            (" 12.00", InvalidFormat),
            ("$12.00", InvalidFormat),
            ("1,234.56", InvalidFormat),
            ("12.345", TooManyDecimals),
            ("92233720368547758.08", OutOfRange),
            ("99999999999999999999", OutOfRange),
        ];

        for (input, expected) in cases {
            assert_eq!(input.parse::<Money>(), Err(expected), "input: {input:?}");
        }
    }

    #[test]
    fn displays_as_dollars() {
        let cases = [
            (0, "$0.00"),
            (5, "$0.05"),
            (-5, "-$0.05"),
            (99999, "$999.99"),
            (123456, "$1,234.56"),
            (-123456, "-$1,234.56"),
            (100_000_000, "$1,000,000.00"),
            (i64::MIN, "-$92,233,720,368,547,758.08"),
        ];

        for (cents, expected) in cases {
            assert_eq!(Money::from_cents(cents).to_string(), expected);
        }
    }

    fn cents_of(amounts: &[Money]) -> Vec<i64> {
        amounts.iter().map(|m| m.cents()).collect()
    }

    #[test]
    fn splits_with_leftover_cents_first() {
        assert_eq!(
            cents_of(&Money::from_cents(10_000).split_evenly(3)),
            [3334, 3333, 3333]
        );
        assert_eq!(
            cents_of(&Money::from_cents(-10_000).split_evenly(3)),
            [-3334, -3333, -3333]
        );
        assert_eq!(
            cents_of(&Money::from_cents(2).split_evenly(5)),
            [1, 1, 0, 0, 0]
        );
        assert_eq!(cents_of(&Money::from_cents(500).split_evenly(1)), [500])
    }

    #[test]
    fn split_never_loses_a_cent() {
        for total in [-1001, -1, 0, 1, 7, 9_999, 10_000] {
            for parts in 1..=12 {
                let pieces = Money::from_cents(total).split_evenly(parts);
                assert_eq!(pieces.len(), parts as usize);
                assert_eq!(pieces.iter().sum::<Money>(), Money::from_cents(total));

                let min = pieces.iter().min().unwrap();
                let max = pieces.iter().max().unwrap();
                assert!(max.cents() - min.cents() <= 1, "uneven: {total} / {parts}");
            }
        }
    }

    #[test]
    #[should_panic(expected = "zero parts")]
    fn split_into_zero_parts_panics() {
        Money::from_cents(100).split_evenly(0);
    }
}
