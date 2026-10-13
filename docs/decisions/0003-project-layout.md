# 0003. Project layout

- **Status:** Accepted
- **Date:** 2026-10-12

## Context

Patina will grow to cover accounts, transactions, categories, budgets
and import. Each of these has pure domain rules, SQL queries and HTTP
handlers. The layout should keep one feature's code together, keep
business rules testable without a database or web server, and stay
cheap to restructure while the app is small.

## Options considered

- **Layered folders** (`models/`, `repos/`, `routes/`): familiar, but
  one feature's code ends up spread across the whole tree.
- **Feature modules in one crate** (modular monolith): each feature
  keeps its model, queries and handlers together. Boundaries are
  enforced by module privacy, which is softer than separate crates.
- **Cargo workspace with one crate per module**: hard compile-time
  boundaries, like separate .NET projects. More setup (a `Cargo.toml`
  per crate and dependency wiring between them) than this app needs
  right now.

## Decision

One crate, organized as a library (`src/lib.rs`) with a thin binary
(`src/main.rs`). Code is grouped by feature, with a shared kernel:

    src/
      main.rs        thin entry point: config, build app, serve
      lib.rs         declares top-level modules
      config.rs
      web.rs         AppState, router assembly, error responses
      kernel/        shared value types (Money, YearMonth, ...)
      <feature>/     accounts, transactions, categories, budgets, import
        model.rs     pure types and rules
        repo.rs      sqlx queries
        routes.rs    Axum handlers
    migrations/      sqlx migrations
    templates/       Askama templates
    static/          CSS, htmx
    tests/           integration tests; fake data in tests/fixtures/

Rules:

- Dependencies flow `routes → repo → model`. `model.rs` depends only
  on `kernel` and the standard library, never on Axum or sqlx.
- `kernel` depends on no feature module. `Money` deriving
  `sqlx::Type` is an accepted exception (see 0002).
- Features use each other only through `pub` items. Anything else is
  private or `pub(crate)`.
- Modules use the `name.rs` + `name/` convention, not `mod.rs`.
- No module is named `core`, which would clash with Rust's built-in
  `core` crate.

The exact list of feature modules is set by the domain-model ADR.

## Consequences

- One feature's code is in one place, and its domain rules can be
  unit tested without I/O.
- Integration tests can import the library (`use patina::...`).
- Module boundaries depend on discipline plus privacy. If they start
  leaking, or compile times grow, the modules can be split into
  workspace crates. That would be a mechanical refactor.
