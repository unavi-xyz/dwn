use sqlx::mysql::MySqlPoolOptions;

#[sqlx::test]
async fn insert_record() {
    if std::env::var("DATABASE_URL").is_err() {
        dotenv::dotenv().ok();
    }

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL");
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
