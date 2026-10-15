# Patina

A self-hosted personal budgeting app, inspired by Monarch.

Patina imports transactions from bank exports (CSV/OFX), categorizes them,
and tracks spending against monthly category budgets with rollover.
It's designed to run on a machine you own, with your data stored locally
in SQLite.

**Status:** early development.

## Building

Requires Rust 1.85+ (the toolchain is stable via `rust-toolchain.toml`).

    cargo build
    cargo run

## Your data

All real financial data — the SQLite database and any bank export files —
lives in `data/`, which is git-ignored. Never commit real data.
Test fixtures use fake data and live under `tests/fixtures/`.

## Design

Architecture decisions are recorded in [`docs/decisions/`](docs/decisions/).

## License

Apache-2.0. See [LICENSE](LICENSE).
