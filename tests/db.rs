use patina::db;

#[tokio::test]
async fn connect_creates_migrates_and_configures() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("patina.db");

    let pool = db::connect(&path).await.unwrap();

    let journal_mode: String = sqlx::query_scalar("PRAGMA journal_mode")
        .fetch_one(&pool)
        .await
        .unwrap();
    let synchronous: i64 = sqlx::query_scalar("PRAGMA synchronous")
        .fetch_one(&pool)
        .await
        .unwrap();
    let foreign_keys: i64 = sqlx::query_scalar("PRAGMA foreign_keys")
        .fetch_one(&pool)
        .await
        .unwrap();
    let account_kinds: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM account_kinds")
        .fetch_one(&pool)
        .await
        .unwrap();

    assert_eq!(journal_mode, "wal");
    assert_eq!(synchronous, 1); // NORMAL
    assert_eq!(foreign_keys, 1);
    assert_eq!(account_kinds, 4);
    pool.close().await;

    // Connecting again finds nothing to migrate.
    let pool = db::connect(&path).await.unwrap();
    pool.close().await;
}
