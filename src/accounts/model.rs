use std::{fmt, str::FromStr};

use crate::kernel::AccountId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AccountKind {
    Checking,
    Savings,
    CreditCard,
    Cash,
}

impl AccountKind {
    pub const ALL: [AccountKind; 4] = [
        AccountKind::Checking,
        AccountKind::Savings,
        AccountKind::CreditCard,
        AccountKind::Cash,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            AccountKind::Checking => "checking",
            AccountKind::Savings => "savings",
            AccountKind::CreditCard => "credit_card",
            AccountKind::Cash => "cash",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseAccountKindError(String);

impl fmt::Display for ParseAccountKindError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown accound kind: {:?}", self.0)
    }
}

impl std::error::Error for ParseAccountKindError {}

impl FromStr for AccountKind {
    type Err = ParseAccountKindError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        AccountKind::ALL
            .into_iter()
            .find(|kind| kind.as_str() == s)
            .ok_or_else(|| ParseAccountKindError(s.to_owned()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AccountName(String);

impl AccountName {
    pub fn new(name: &str) -> Option<Self> {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(AccountName(trimmed.to_owned()))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for AccountName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewAccount {
    pub name: AccountName,
    pub kind: AccountKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Account {
    pub id: AccountId,
    pub name: AccountName,
    pub kind: AccountKind,
    pub archived: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kinds_round_trip_through_text() {
        for kind in AccountKind::ALL {
            assert_eq!(kind.as_str().parse::<AccountKind>(), Ok(kind));
        }
    }

    #[test]
    fn rejects_unknown_kinds() {
        for input in ["", "Checking", "credit card", "loan"] {
            assert_eq!(
                input.parse::<AccountKind>(),
                Err(ParseAccountKindError(input.to_owned())),
                "input: {input:?}"
            );
        }
    }

    #[test]
    fn names_are_trimmed() {
        let name = AccountName::new("  Joint Checking\n").unwrap();
        assert_eq!(name.as_str(), "Joint Checking");
    }

    #[test]
    fn rejects_blank_names() {
        for input in ["", "  ", "\t\n"] {
            assert_eq!(AccountName::new(input), None, "input: {input:?}");
        }
    }
}
