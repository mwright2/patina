//! Typed IDs over SQLite rowids.
//!
//! Mixing ID types is a compile error:
//!
//! ```compile_fail
//! use patina::kernel::{AccountId, CategoryId};
//! fn takes_account(_: AccountId) {}
//! takes_account(CategoryId::new(1));
//! ```

use std::fmt;

macro_rules! id_type {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, sqlx::Type)]
        #[sqlx(transparent)]
        pub struct $name(i64);

        impl $name {
            pub const fn new(value: i64) -> Self {
                $name(value)
            }

            pub const fn get(self) -> i64 {
                self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

id_type!(AccountId);
id_type!(CategoryGroupId);
id_type!(CategoryId);
id_type!(TransactionId);
