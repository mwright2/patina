-- Reference tables for stored enums (0007).
CREATE TABLE account_kinds (kind TEXT PRIMARY KEY) STRICT, WITHOUT ROWID;
INSERT INTO account_kinds (kind) VALUES ('checking'), ('savings'), ('credit_card'), ('cash');

CREATE TABLE group_kinds (kind TEXT PRIMARY KEY) STRICT, WITHOUT ROWID;
INSERT INTO group_kinds (kind) VALUES ('income'), ('expense'), ('transfer');

CREATE TABLE accounts (
    id       INTEGER PRIMARY KEY,
    name     TEXT    NOT NULL COLLATE NOCASE CHECK (name <> ''),
    kind     TEXT    NOT NULL REFERENCES account_kinds (kind) ON DELETE RESTRICT,
    archived INTEGER NOT NULL DEFAULT 0 CHECK (archived IN (0, 1))
) STRICT;
CREATE UNIQUE INDEX accounts_active_name ON accounts (name) WHERE archived = 0;

CREATE TABLE category_groups (
    id       INTEGER PRIMARY KEY,
    name     TEXT    NOT NULL COLLATE NOCASE CHECK (name <> ''),
    kind     TEXT    NOT NULL REFERENCES group_kinds (kind) ON DELETE RESTRICT,
    archived INTEGER NOT NULL DEFAULT 0 CHECK (archived IN (0, 1))
) STRICT;
CREATE UNIQUE INDEX category_groups_active_name ON category_groups (name) WHERE archived = 0;

CREATE TABLE categories (
    id       INTEGER PRIMARY KEY,
    group_id INTEGER NOT NULL REFERENCES category_groups (id) ON DELETE RESTRICT,
    name     TEXT    NOT NULL COLLATE NOCASE CHECK (name <> ''),
    archived INTEGER NOT NULL DEFAULT 0 CHECK (archived IN (0, 1))
) STRICT;
CREATE INDEX categories_group ON categories (group_id);
CREATE UNIQUE INDEX categories_active_name ON categories (name) WHERE archived = 0;

CREATE TABLE transactions (
    id         INTEGER PRIMARY KEY,
    account_id INTEGER NOT NULL REFERENCES accounts (id) ON DELETE RESTRICT,
    posted_on  TEXT    NOT NULL CHECK (posted_on IS date(posted_on)),
    payee      TEXT    NOT NULL,
    amount     INTEGER NOT NULL
) STRICT;
CREATE INDEX transactions_account_date ON transactions (account_id, posted_on);
CREATE INDEX transactions_posted_on ON transactions (posted_on);

CREATE TABLE transaction_lines (
    transaction_id INTEGER NOT NULL REFERENCES transactions (id) ON DELETE CASCADE,
    position       INTEGER NOT NULL CHECK (position >= 0),
    amount         INTEGER NOT NULL,
    category_id    INTEGER REFERENCES categories (id) ON DELETE RESTRICT,
    PRIMARY KEY (transaction_id, position)
) STRICT, WITHOUT ROWID;
CREATE INDEX transaction_lines_category ON transaction_lines (category_id);
