# 0001. Tech stack

- **Status:** Accepted
- **Date:** 2026-10-05

## Context

Patina is a single-household budgeting app, self-hosted on a personal PC.
It's also a learning project: practicing web app design from scratch
and learning Rust along the way. The UI is mostly tables, forms, and charts.

## Options considered

- **ASP.NET Core + Razor Pages + HTMX + SQLite/Dapper** — familiar, fastest
  path to a finished app, but less new learning.
- **Rust (Axum + sqlx + Askama + HTMX) + SQLite** — strong type system for
  domain modeling, compile-time-checked SQL, single small binary to deploy;
  slower progress while learning the language.
- **SPA (React/SvelteKit) + API** — closest to Monarch's architecture, but
  doubles the surface area for no real benefit at single-user scale.

## Decision

Rust: Axum (web), sqlx (database), Askama (templates), HTMX (interactivity),
SQLite (storage). Server-rendered, no SPA.

## Consequences

- Money, IDs, and transaction kinds can be modeled with newtypes and enums,
  making whole classes of bugs unrepresentable.
- Deployment is a single binary plus a SQLite file.
- Development will be slower initially while learning Rust/async.
- Transactions will come in via file import first (CSV/OFX); a SimpleFIN
  sync source may be added later behind the same import abstraction.
