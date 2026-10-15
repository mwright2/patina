use sqlx::{SqlitePool, error::ErrorKind};

const SEED: &str = "
    INSERT INTO accounts (id, name, kind) VALUES (1, 'Checking', 'checking');
    INSERT INTO category_groups (id, name, kind) VALUES (1, 'Food', 'expense');
    INSERT INTO category_groups (id, name, kind) VALUES (2, 'Home', 'expense');
    INSERT INTO categories (id, group_id, name) VALUES (1, 1, 'Groceries');
    INSERT INTO transactions (id, account_id, posted_on, payee, amount)
        VALUES (1, 1, '2026-10-15', 'Grocer', -4250);
    INSERT INTO transaction_lines (transaction_id, position, amount, category_id)
        VALUES (1, 0, -4250, 1);
";

async fn seed(pool: &SqlitePool) {
    sqlx::raw_sql(SEED).execute(pool).await.unwrap();
}

async fn failure(pool: &SqlitePool, sql: &'static str) -> ErrorKind {
    let err = sqlx::raw_sql(sql)
        .execute(pool)
        .await
        .expect_err("statement should fail");
    err.as_database_error()
        .expect("should be a database error")
        .kind()
}

#[sqlx::test]
async fn rejects_unknown_enum_values(pool: SqlitePool) {
    let kind = failure(
        &pool,
        "INSERT INTO accounts (name, kind) VALUES ('Car loan', 'loan')",
    )
    .await;
    assert!(matches!(kind, ErrorKind::ForeignKeyViolation), "{kind:?}");
}

#[sqlx::test]
async fn strict_tables_reject_wrong_types(pool: SqlitePool) {
    seed(&pool).await;
    // A datatype error has no specific ErrorKind; failing is enough.
    failure(
        &pool,
        "UPDATE transactions SET amount = 'lots' WHERE id = 1",
    )
    .await;
}

#[sqlx::test]
async fn rejects_malformed_dates(pool: SqlitePool) {
    seed(&pool).await;
    for sql in [
        "INSERT INTO transactions (account_id, posted_on, payee, amount) VALUES (1, '2026-13-01', 'X', -1)",
        "INSERT INTO transactions (account_id, posted_on, payee, amount) VALUES (1, '2026-02-30', 'X', -1)",
        "INSERT INTO transactions (account_id, posted_on, payee, amount) VALUES (1, '10/15/2026', 'X', -1)",
        "INSERT INTO transactions (account_id, posted_on, payee, amount) VALUES (1, '2026-10-15 09:30:00', 'X', -1)",
    ] {
        let kind = failure(&pool, sql).await;
        assert!(matches!(kind, ErrorKind::CheckViolation), "{sql}: {kind:?}");
    }
}

#[sqlx::test]
async fn names_are_unique_among_active_rows_ignoring_case(pool: SqlitePool) {
    seed(&pool).await;
    let kind = failure(
        &pool,
        "INSERT INTO categories (group_id, name) VALUES (2, 'GROCERIES')",
    )
    .await;
    assert!(matches!(kind, ErrorKind::UniqueViolation), "{kind:?}");

    sqlx::raw_sql(
        "UPDATE categories SET archived = 1 WHERE id = 1;
         INSERT INTO categories (group_id, name) VALUES (2, 'Groceries');",
    )
    .execute(&pool)
    .await
    .unwrap();
}

#[sqlx::test]
async fn restricts_deleting_referenced_rows(pool: SqlitePool) {
    seed(&pool).await;
    for sql in [
        "DELETE FROM accounts WHERE id = 1",
        "DELETE FROM category_groups WHERE id = 1",
        "DELETE FROM categories WHERE id = 1",
    ] {
        let err = sqlx::raw_sql(sql).execute(&pool).await.expect_err(sql);
        let message = err.as_database_error().expect("database error").message();
        assert_eq!(message, "FOREIGN KEY constraint failed", "{sql}");
    }
}

#[sqlx::test]
async fn deleting_a_transaction_deletes_its_lines(pool: SqlitePool) {
    seed(&pool).await;
    sqlx::raw_sql("DELETE FROM transactions WHERE id = 1")
        .execute(&pool)
        .await
        .unwrap();

    let lines: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM transaction_lines")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(lines, 0);
}
