use std::fmt;

use chrono::NaiveDate;

use crate::kernel::{AccountId, CategoryId, Money, TransactionId, YearMonth};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Line {
    pub amount: Money,
    pub category_id: Option<CategoryId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Allocation {
    amount: Money,
    lines: Vec<Line>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllocationError {
    NoLines,
    Unbalanced { amount: Money, allocated: Money },
}

impl fmt::Display for AllocationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AllocationError::NoLines => f.write_str("a transaction needs at least one line"),
            AllocationError::Unbalanced { amount, allocated } => {
                write!(f, "lines total {allocated}, but the amount is {amount}")
            }
        }
    }
}

impl std::error::Error for AllocationError {}

impl Allocation {
    pub fn single(amount: Money, category_id: Option<CategoryId>) -> Self {
        Allocation {
            amount,
            lines: vec![Line {
                amount,
                category_id,
            }],
        }
    }

    pub fn split(amount: Money, lines: Vec<Line>) -> Result<Self, AllocationError> {
        if lines.is_empty() {
            return Err(AllocationError::NoLines);
        }

        let allocated: Money = lines.iter().map(|line| line.amount).sum();
        if allocated != amount {
            return Err(AllocationError::Unbalanced { amount, allocated });
        }

        Ok(Allocation { amount, lines })
    }

    pub fn amount(&self) -> Money {
        self.amount
    }

    pub fn lines(&self) -> &[Line] {
        &self.lines
    }

    pub fn is_split(&self) -> bool {
        self.lines.len() > 1
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewTransaction {
    pub account_id: AccountId,
    pub date: NaiveDate,
    pub payee: String,
    pub allocation: Allocation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transaction {
    pub id: TransactionId,
    pub account_id: AccountId,
    pub date: NaiveDate,
    pub payee: String,
    pub allocation: Allocation,
}

impl Transaction {
    pub fn month(&self) -> YearMonth {
        YearMonth::of(self.date)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dollars(cents: i64) -> Money {
        Money::from_cents(cents)
    }

    fn line(cents: i64, category: Option<i64>) -> Line {
        Line {
            amount: dollars(cents),
            category_id: category.map(CategoryId::new),
        }
    }

    #[test]
    fn single_is_one_balanced_line() {
        let allocation = Allocation::single(dollars(-4_250), None);
        assert_eq!(allocation.amount(), dollars(-4_250));
        assert_eq!(allocation.lines(), [line(-4_250, None)]);
        assert!(!allocation.is_split());
    }

    #[test]
    fn accepts_balanced_split() {
        let lines = vec![line(-6_000, Some(1)), line(-4_000, Some(2))];
        let allocation = Allocation::split(dollars(-10_000), lines).unwrap();
        assert_eq!(allocation.lines().len(), 2);
        assert!(allocation.is_split());
    }

    #[test]
    fn accepts_mixed_signs() {
        let lines = vec![line(-12_000, Some(1)), line(2_000, Some(2))];
        assert!(Allocation::split(dollars(-10_000), lines).is_ok());
    }

    #[test]
    fn rejects_no_lines() {
        assert_eq!(
            Allocation::split(dollars(-100), vec![]),
            Err(AllocationError::NoLines)
        );
    }

    #[test]
    fn rejects_unbalanced_split() {
        let lines = vec![line(-6_000, Some(1)), line(-3_000, Some(2))];
        assert_eq!(
            Allocation::split(dollars(-10_000), lines),
            Err(AllocationError::Unbalanced {
                amount: dollars(-10_000),
                allocated: dollars(-9_000),
            })
        );
    }

    #[test]
    fn belongs_to_month_of_its_date() {
        let txn = Transaction {
            id: TransactionId::new(1),
            account_id: AccountId::new(1),
            date: NaiveDate::from_ymd_opt(2026, 10, 31).unwrap(),
            payee: "Grocer".to_owned(),
            allocation: Allocation::single(dollars(-500), None),
        };
        assert_eq!(txn.month(), YearMonth::new(2026, 10).unwrap());
    }
}
