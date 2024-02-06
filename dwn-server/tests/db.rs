use sqlx::{MySql, Pool};
use tracing_test::traced_test;

#[sqlx::test]
#[traced_test]
async fn insert_record() {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL is not set");

    let pool = Pool::<MySql>::connect(database_url.as_str())
        .await
        .expect("Failed to connect to database");

    // Clear the table
    sqlx::query!("DELETE FROM Record")
        .execute(&pool)
        .await
        .expect("Failed to clear table");

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
