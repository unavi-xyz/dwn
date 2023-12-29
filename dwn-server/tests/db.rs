use sqlx::{MySql, Pool};
use tracing_test::traced_test;

#[sqlx::test]
#[traced_test]
#[cfg(not(feature = "disable-sqlx-testing"))]
async fn insert_record(pool: Pool<MySql>) {
    sqlx::query!("INSERT INTO Record (id, data) VALUES ('test', 'test')")
        .execute(&pool)
        .await
        .expect("Failed to insert record");

    let record = sqlx::query!("SELECT * FROM Record WHERE id = 'test'")
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch record");

    assert_eq!(record.id, "test");
}
