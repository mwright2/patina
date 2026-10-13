# 0002. Money representation

- **Status:** Accepted
- **Date:** 2026-10-05

## Context

Every part of Patina handles money: imported transactions, budgets,
rollover, and reports. The representation must be exact, because
small rounding errors add up across sums. It must also work well with
SQLite, which has no decimal type. Patina tracks a single household's
finances in US dollars.

This ADR defines what a money value *is*. What a signed amount *means*
(for example, whether an expense is stored as a negative number) is a
domain-model question and is decided in 0004.

## Options considered

- **`f64`**: simple, and stored as SQLite `REAL`. Not exact
  (`0.1 + 0.2 != 0.3`), so errors build up in totals. Rejected.
- **`rust_decimal`**: exact, and allows any number of decimal places.
  SQLite has no decimal type, so values would be stored as `TEXT`.
  `SUM()` in SQL would then be wrong or impossible, and every total
  would have to be added up in Rust.
- **`i64` cents in a newtype**: exact, fast, and stored as SQLite
  `INTEGER`, so `SUM()` in SQL is exact. Always two decimal places,
  with a range of roughly ±$92 quadrillion.

## Decision

Money is a `Money` newtype wrapping `i64` cents. There is one
currency, USD, and it is implicit: no currency field and no currency
column.

- **Allowed operations:** `Money + Money`, `Money - Money`, `-Money`,
  summing a list of amounts, comparisons, and `Money * i64`.
  `Money * Money` and `Money / Money` are not provided.
- **Splitting:** an allocation method divides an amount into *n*
  parts and gives the leftover cents to the first parts, so no cent
  is lost. For example, $100 split three ways is $33.34, $33.33, $33.33.
- **Overflow:** the release profile sets `overflow-checks = true`, so
  overflow panics instead of silently wrapping.
- **Parsing:** `Money` parses one plain format only, such as
  `-1234.56`, and rejects more than two decimal places. Each import
  adapter converts its bank's format (`$1,234.56`, `(12.30)`, and so
  on) to that form before parsing.
- **Display:** `Display` prints `-$1,234.56`. Styling, such as color
  for negative amounts, belongs in templates.
- **Storage:** `#[sqlx(transparent)]` stores the value as a plain
  `INTEGER` column.
- **Sign:** `Money` may be negative. What the sign means is decided
  in 0004.

## Consequences

- Totals and budget calculations can be done in SQL and are exact.
- The type system prevents mixing money with plain numbers by mistake.
- Supporting more than one currency later would need a migration and
  a currency field on `Money`.
- Amounts with more than two decimal places, such as fractional cents
  from a bank export, are rejected at import and must be handled
  there.
