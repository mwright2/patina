# 0004. Domain model

- **Status:** Accepted
- **Date:** 2026-10-13

## Context

Patina needs a domain model for accounts, transactions, categories,
category groups and split transactions. The model must support
Monarch-style monthly category budgets with rollover. Budgets are
decided in a later ADR, but the model must make "how much was spent
in category X in month M" a simple, exact query.

Transactions come from bank exports (0001), and amounts are `Money`
(0002), which may be negative. This ADR decides what the sign means,
and it sets the list of feature modules that 0003 left open.

This household rarely moves money between accounts it tracks, so
transfers are deferred. Credit card payments must still be kept out
of budget totals.

## Options considered

**Splits**

- **Category on the transaction, plus optional split rows**: simple
  for unsplit transactions, but every spending query needs two code
  paths combined with a UNION.
- **Every transaction has one or more lines, and the category is on
  the lines**: one query path for all spending. Costs an extra row
  per transaction, plus an invariant (lines sum to the amount) that
  SQLite can't easily enforce.
- **Splits as child transactions of a hidden parent (Monarch)**: every
  query needs a "not a split parent" filter, and some stored
  transactions aren't real bank transactions.

**Uncategorized transactions**

- **Nullable category**: the compiler forces code to handle the
  uncategorized case.
- **A sentinel "Uncategorized" category**: appears in every category
  list and budget and has to be filtered out.

**Category hierarchy**

- **Two levels (group → category)**: matches Monarch and maps directly
  to budgets.
- **Arbitrary tree (`parent_id`)**: more flexible, but rollups need
  recursive queries.

**Storing enums** (0003 keeps sqlx out of models)

- **Map in the repo**: models stay free of sqlx, at the cost of a
  small amount of conversion code.
- **Derive `sqlx::Type` in models**: less code, but adds exceptions to
  0003's rule.

**Dates**

- **`chrono::NaiveDate`, allowed in models by amending 0003**.
- **A kernel `Date` newtype wrapping chrono**: keeps 0003's rule as
  written, at the cost of extra ceremony.

## Decision

### Modules

Three feature modules: `accounts`, `categories` (groups and
categories) and `transactions` (transactions and their lines).
`budgets` and `import` come in later ADRs.

### Accounts

An account has an ID, a name, a kind (`Checking`, `Savings`,
`CreditCard`, `Cash`) and an `archived` flag. The kind affects display
only. Balances are not stored or modeled yet; how to anchor them
(opening balance or statement balance) is decided with import.

### Transactions

- A transaction has an ID, an account, a date, a payee, a signed
  amount and one or more lines.
- **Sign:** negative is money out, positive is money in, matching bank
  exports. A credit card purchase is negative, so a card's balance is
  negative (a liability).
- **Date:** the posted date with no time of day, as
  `chrono::NaiveDate`. A transaction belongs to the month of its date.
- **Lines:** each line has an amount and an optional category. An
  unsplit transaction has one line. Lines have no ID in the domain;
  saving a transaction replaces its lines.
- **Invariant:** a transaction's amount and lines are held together
  in an `Allocation`. `Allocation::split` returns an error if there
  are no lines or if they don't sum to the amount; `Allocation::single`
  builds the unsplit case and can't fail. Its fields are private, so
  every transaction, including one loaded from the database, has a
  balanced allocation. Transactions themselves are plain data with
  public fields.
- **Uncategorized:** a line's category is `Option<CategoryId>`. `None`
  (NULL) means not yet categorized.
- Transactions will carry an optional external ID for de-duplicating
  imports. The details are decided with import.

### Categories and groups

- Two levels: each category belongs to exactly one group.
- A group has a kind, which its categories inherit: `Income`,
  `Expense` or `Transfer`.
- `Transfer` categories (for example, "Credit Card Payment") are
  excluded from budget and income totals.
- Categories and groups are archived, never deleted. Foreign keys use
  `ON DELETE RESTRICT`, and the `foreign_keys` pragma is set explicitly
  when connecting.
- Expense categories may have positive amounts, because refunds need
  them. Spending in a category is the negated sum of its lines, so a
  refund reduces spending.

### Transfers

Linking the two sides of a transfer is deferred. Until then, both
sides are categorized under a `Transfer` category.

### IDs and kernel types

- IDs are `i64` newtypes over SQLite rowids: `AccountId`,
  `CategoryGroupId`, `CategoryId` and `TransactionId`.
- IDs and a new `YearMonth` type live in `kernel`. IDs derive
  `sqlx::Type` with `#[sqlx(transparent)]`. This extends the exception
  0003 made for `Money` to kernel value types in general.

### Changes to 0003's rules

- Models may depend on pure value crates that do no I/O, such as
  chrono, in addition to `kernel` and the standard library.
- Models still never depend on sqlx or Axum. Enums in models provide
  `as_str()` and `FromStr`; repos store them as `TEXT` and convert.
  Repos read rows into private row structs and build domain types
  through their constructors.

## Consequences

- Spending per category per month is one query over lines joined to
  transactions, which the budgets ADR can build on.
- Every transaction writes at least two rows. The split invariant is
  enforced in Rust, not in the database, so code that bypasses the
  repo could store unbalanced lines.
- Repos need row structs and mapping code, much like Dapper DTOs
  mapped to entities.
- Archived categories accumulate, and the UI will need to hide them.
- Credit card payments must be categorized as transfers, or budgets
  will count the spending twice.
- Account balances and transfer linking are not available until later
  ADRs.
- Some bank CSVs reverse the sign; the import ADR must handle that per
  account.
