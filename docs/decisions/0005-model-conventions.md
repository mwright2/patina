# 0005. Model conventions

- **Status:** Accepted
- **Date:** 2026-10-15

## Context

Implementing the models from 0004 raised patterns that every feature
module should follow, and one rule about categories that 0004 didn't
cover. Recording them here keeps future features consistent.

## Options considered

**Records that haven't been saved yet** (IDs are rowids assigned on insert)

- **`Option<Id>` on the record**: one type, but every saved record
  carries a `None` case that can't happen.
- **A placeholder ID before insert**: the type system can't tell saved
  from unsaved.
- **Separate `NewX` and `X` types**: one more type per entity, but
  saved and unsaved records can't be confused.

**Where validation lives**

- **Private fields and a validating constructor on each record**: rules
  get duplicated across `NewX` and `X`, and every edit needs a method.
- **Value types that are valid by construction, with public fields on
  records**: each rule lives in one type, and records are plain data.

**Names**

- **A name type per feature**: prevents mixing names, which isn't a
  real source of bugs.
- **One shared `Name` type**: one rule, one set of tests.

**Moving a category to another group**

- **Allow any move**: moving a category to a group of a different kind
  silently reclassifies its history (for example, pulling it out of
  past budget totals).
- **Only between groups of the same kind**.
- **Never**: safe, but reorganizing groups becomes tedious.

## Decision

- **Unsaved vs saved:** each entity has a `NewX` type without an ID and
  an `X` type with one. A repo's insert takes `NewX` and returns the ID.
- **Valid by construction:** rules live in value types with private
  fields and checked constructors (`Money`, `Name`, `YearMonth`,
  `Allocation`). Records built from them are plain data with public
  fields. A record gets private fields only if it has a rule spanning
  several fields that can't be grouped into one value type.
- **Names:** `kernel::Name` is trimmed and never blank, and is used for
  accounts, category groups and categories. Payees are plain `String`;
  cleaning them up belongs to import.
- **Stored enums:** each has `ALL`, `as_str()` and `FromStr`, and
  repos store `as_str()` as `TEXT`. Those strings are a database
  contract, so renaming one needs a migration. If a third such enum
  appears, generate the pattern with a macro.
- **Allocation lines** may have mixed signs and zero amounts; only
  their total must match the transaction amount.
- **Category moves:** a category may move only to a group of the same
  kind. This is enforced where categories are updated.

## Consequences

- Code that receives an `X` knows it was saved, without `Option`
  checks.
- Records are easy to edit. For example, recategorizing a transaction
  is assigning a new `Allocation`.
- Changing a category's kind means creating a new category and
  recategorizing its transactions.
- The stored-enum pattern is duplicated until a macro replaces it.
- Name uniqueness and display order for categories are not decided;
  they belong with the schema.
