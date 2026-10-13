use patina::kernel::Money;
use sqlx::sqlite::SqlitePool;

async fn memory_db() -> SqlitePool {
    SqlitePool::connect("sqlite::memory:").await.unwrap()
}

#[tokio::test]
async fn round_trips_as_integer() {
    let db = memory_db().await;
    let amount = Money::from_cents(-123_456);

    let (back, column_type): (Money, String) = sqlx::query_as("SELECT ?1, typeof(?1)")
        .bind(amount)
        .fetch_one(&db)
        .await
        .unwrap();

    assert_eq!(back, amount);
    assert_eq!(column_type, "integer");
}

#[tokio::test]
async fn sums_exactly_in_sql() {
    let db = memory_db().await;
    sqlx::query("CREATE TABLE t (amount INTEGER NOT NULL)")
        .execute(&db)
        .await
        .unwrap();

    for cents in [10, 20, -5, 1_000_001] {
        sqlx::query("INSERT INTO t (amount) VALUES (?)")
            .bind(Money::from_cents(cents))
            .execute(&db)
            .await
            .unwrap();
    }

    let total: Money = sqlx::query_scalar("SELECT SUM(amount) FROM t")
        .fetch_one(&db)
        .await
        .unwrap();

    assert_eq!(total, Money::from_cents(1_000_026));
}
