use std::{fmt, str::FromStr};

use crate::kernel::{CategoryGroupId, CategoryId, Name};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GroupKind {
    Income,
    Expense,
    Transfer,
}

impl GroupKind {
    pub const ALL: [GroupKind; 3] = [GroupKind::Income, GroupKind::Expense, GroupKind::Transfer];

    pub const fn as_str(self) -> &'static str {
        match self {
            GroupKind::Income => "income",
            GroupKind::Expense => "expense",
            GroupKind::Transfer => "transfer",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseGroupKindError(String);

impl fmt::Display for ParseGroupKindError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown group kind: {:?}", self.0)
    }
}

impl std::error::Error for ParseGroupKindError {}

impl FromStr for GroupKind {
    type Err = ParseGroupKindError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        GroupKind::ALL
            .into_iter()
            .find(|kind| kind.as_str() == s)
            .ok_or_else(|| ParseGroupKindError(s.to_owned()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewCategoryGroup {
    pub name: Name,
    pub kind: GroupKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CategoryGroup {
    pub id: CategoryGroupId,
    pub name: Name,
    pub kind: GroupKind,
    pub archived: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewCategory {
    pub group_id: CategoryGroupId,
    pub name: Name,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Category {
    pub id: CategoryId,
    pub group_id: CategoryGroupId,
    pub name: Name,
    pub archived: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kinds_round_trip_through_text() {
        for kind in GroupKind::ALL {
            assert_eq!(kind.as_str().parse::<GroupKind>(), Ok(kind));
        }
    }

    #[test]
    fn rejects_unknown_kinds() {
        for input in ["", "Income", "expenses", "savings"] {
            assert_eq!(
                input.parse::<GroupKind>(),
                Err(ParseGroupKindError(input.to_owned())),
                "input: {input:?}"
            );
        }
    }
}
