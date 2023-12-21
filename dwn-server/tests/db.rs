use sqlx::mysql::MySqlPoolOptions;

#[tokio::test]
async fn insert_record() {
    dotenv::dotenv().ok();

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = MySqlPoolOptions::new().connect(&db_url).await.unwrap();

    sqlx::query!("INSERT INTO Record (id, name) VALUES (1, 'test')")
        .execute(&pool)
        .await
        .unwrap();

    let record = sqlx::query!("SELECT * FROM Record WHERE id = 1")
        .fetch_one(&pool)
        .await
        .unwrap();

    assert_eq!(record.ID, Some(1));
}
