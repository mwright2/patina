# 0007. Schema conventions

- **Status:** Accepted
- **Date:** 2026-10-15

## Context

The first migration creates tables for the model in 0004 and 0005.
SQLite is loosely typed by default, can't alter a CHECK constraint,
and has no date or boolean types. Conventions decided now apply to
this migration and every later one.

## Options considered

**Column types**

- **Default (flexible) typing**: any value fits in any column.
- **`STRICT` tables**: SQLite enforces declared types; only `INTEGER`,
  `REAL`, `TEXT`, `BLOB` and `ANY` are allowed.

**Stored enums** (0005 stores `as_str()` as `TEXT`)

- **No constraint**: bad values are only caught on read.
- **`CHECK (kind IN (...))`**: simple, but adding a value means
  rebuilding the table, with foreign keys off if other tables
  reference it.
- **A reference table per enum**: adding a value is an `INSERT`.

**Name uniqueness**

- **None**: duplicates are confusing in pickers and import rules.
- **Unique across all rows**: an archived name can never be reused.
- **Unique among active rows, ignoring case**: names can be reused
  after archiving; unarchiving can conflict.

**Transaction amount**

- **Stored on the transaction**: keeps the bank's figure; could drift
  from the lines.
- **Derived as the sum of the lines**: can't drift; every balance
  query joins lines.

**Line identity**

- **A rowid per line**: an ID the domain doesn't have (0004).
- **`(transaction_id, position)`**: keeps line order and matches the
  domain.

**Dates**

- **ISO 8601 text**: readable, sorts correctly, works with SQLite's
  date functions.
- **Julian day or `yyyymmdd` integers**: compact, but unreadable
  and need conversion everywhere.

**Display order**

- **Alphabetical**.
- **A `position` column**: supports manual ordering, plus UI to
  manage it.

## Decision

- **Strict tables:** every table is `STRICT`. Booleans are `INTEGER`
  with `CHECK (x IN (0, 1))`. Money is `INTEGER` cents (0002). IDs
  are `INTEGER PRIMARY KEY` (the rowid) without `AUTOINCREMENT`.
- **Stored enums:** each has a reference table (`account_kinds`,
  `group_kinds`) with a single `kind TEXT PRIMARY KEY`, seeded by
  migrations and referenced by foreign key. Adding a value is an
  `INSERT` migration. Renaming or removing one needs a data
  migration, as 0005 already says.
- **Names:** `TEXT NOT NULL COLLATE NOCASE CHECK (name <> '')`,
  unique among non-archived rows through a partial unique index.
  Category names are unique across all groups. `Name` (0005) stays
  the real rule; the CHECK is a backstop.
- **Foreign keys** use `ON DELETE RESTRICT`, except lines, which
  cascade when their transaction is deleted. Every foreign key column
  is the leftmost column of some index, except references to
  reference tables, whose rows are never deleted.
- **Transactions store their amount.** Repos rebuild every
  transaction through `Allocation::split(amount, lines)`, so lines
  that don't add up surface as an error on load.
- **Lines** have primary key `(transaction_id, position)`, where
  `position` is the line's index in the allocation, starting at 0.
  The table is `WITHOUT ROWID`. Saving a transaction replaces all its
  lines (0004).
- **Dates** are `TEXT` in `YYYY-MM-DD` form, as sqlx's `chrono`
  feature encodes `NaiveDate`. The posted date column is `posted_on`,
  checked with `CHECK (posted_on IS date(posted_on))`. It uses `IS`
  because a CHECK that evaluates to NULL passes. A month is queried
  as `posted_on >= first day AND posted_on < next month's first day`.
- **Display order** is alphabetical by name for now. A `position`
  column can be added with `ADD COLUMN` when manual ordering is
  needed.
- **Naming:** plural snake_case tables, foreign keys named
  `<singular>_id`, indexes named `<table>_<purpose>`.

## Consequences

- SQLite itself rejects wrongly typed values.
- SQLite tools older than 3.37 can't open the database.
- Changing a CHECK or most column definitions later requires
  SQLite's table-rebuild procedure; `ADD COLUMN` and new indexes are
  cheap.
- `query!` infers `posted_on` as `String`, so queries need a type
  override (`posted_on AS "posted_on: NaiveDate"`).
- `COLLATE NOCASE` only folds ASCII letters.
- The balanced-lines invariant and the same-kind category move rule
  (0005) remain enforced in Rust, not in the database.
- The external ID for de-duplicating imports is added in the import
  ADR, as a new column plus a unique index.
- SQLite reports `ON DELETE RESTRICT` violations as trigger errors, so
  sqlx gives them `ErrorKind::Other` rather than
  `ForeignKeyViolation`. Code that needs to recognize them must check
  the message. Referenced rows are archived rather than deleted, so
  only bugs should hit this.
