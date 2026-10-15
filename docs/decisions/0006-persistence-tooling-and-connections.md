# 0006. Persistence tooling and connections

- **Status:** Accepted
- **Date:** 2026-10-15

## Context

Patina is about to get its first migration and its first repos. That
needs decisions on how migrations are written and applied, how SQL is
checked, and how connections to SQLite are configured. The database
holds real financial data on a personal PC (0001), and the app ships
as a single binary.

## Options considered

**Applying migrations**

- **Automatically at startup** (`sqlx::migrate!()` embedded in the
  binary): nothing to forget, but a bad migration runs against real
  data as soon as the app starts.
- **An explicit `migrate` subcommand**: deliberate, but one more step.
- **sqlx-cli only**: the CLI would be needed wherever the app runs.

**Rolling back**

- **Reversible migrations** (up and down scripts): twice the SQL, and
  down scripts are rarely tested.
- **Forward-only, with backups**: simpler; recovery means restoring a
  copy.

**Checking SQL**

- **Runtime queries** (`query_as::<_, T>()`): no tooling, but errors
  only show up when tests run.
- **Compile-time macros against a live database**: every build needs
  a migrated dev database.
- **Compile-time macros with offline data** (`.sqlx/`, committed):
  builds work without a database; needs sqlx-cli and a `prepare` step.

**Journal mode**

- **Rollback journal** (SQLite's default): readers and the writer
  block each other.
- **WAL**: readers don't block the writer; adds `-wal` and `-shm`
  files and changes how backups are taken.

## Decision

- **Migrations** are SQL files in `migrations/`, created with
  `sqlx migrate add -s <name>` (sequential numbering). They are
  embedded with `sqlx::migrate!()` and applied by `db::connect`
  before it returns the pool, so the app never runs against an old
  schema. Migrations are forward-only. An applied migration is never
  edited (sqlx checksums them); during development, recreate the dev
  database instead. `build.rs` makes Cargo recompile when
  `migrations/` changes.
- **sqlx-cli is a development tool only**: adding migrations,
  creating the dev database and preparing offline query data. The
  deployed app needs only the binary.
- **Queries in repos** use `query!`, `query_as!` and `query_scalar!`.
  Offline data in `.sqlx/` is committed. `DATABASE_URL` in `.env`
  points at a throwaway `dev.db`, never at `data/`. Runtime queries
  (`query`, `raw_sql`) are fine in tests and for SQL that must be
  built dynamically.
- **Tests** that touch the schema or repos use `#[sqlx::test]`, which
  gives each test a fresh, migrated database.
- **Connections:** `db::connect(path)` opens a pool of at most 4
  connections with `create_if_missing`, WAL journal mode,
  `synchronous = NORMAL`, `foreign_keys = ON` (set explicitly even
  though sqlx enables it by default) and a 5-second busy timeout.
- **Database path:** from `PATINA_DATABASE`, defaulting to
  `data/patina.db`. It is deliberately separate from `DATABASE_URL`,
  so dev tooling and the real database never point at the same file.
- **Layout:** amends 0003 with `src/db.rs` (connection and
  migrations) and `build.rs`.
- **Toolchain:** `rust-version` is the toolchain we build with, currently 1.99.

## Consequences

- Schema and query mismatches are compile errors. Changing a column
  breaks the build until the queries are fixed and `prepare` is run.
- sqlx-cli must be installed and match sqlx 0.9. A forgotten
  `prepare` leaves `.sqlx/` stale; `cargo sqlx prepare --check` finds
  that and belongs in CI if CI is added.
- Migrations run against real data on startup. Taking a backup with
  `VACUUM INTO` under `data/backups/` before applying pending
  migrations is a follow-up.
- With WAL, a plain file copy is not a safe backup; use `VACUUM INTO`
  or the SQLite backup API.
- With `synchronous = NORMAL`, a power cut can lose the last few
  commits but does not corrupt the database.
- SQLite allows one writer at a time whatever the pool size. A
  transaction that reads and then writes can fail with "database is
  busy" without waiting; repos decide how to handle that (for
  example, `BEGIN IMMEDIATE`).
