use sqlx::{MySql, Pool};

#[cfg(not(feature = "disable-sqlx-testing"))]
#[sqlx::test]
async fn insert_record(pool: Pool<MySql>) {
    sqlx::query!("INSERT INTO Record (id, name) VALUES (1, 'test')")
        .execute(&pool)
        .await
        .expect("Failed to insert record");

    let record = sqlx::query!("SELECT * FROM Record WHERE id = 1")
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch record");

    assert_eq!(record.Id, Some(1));
}
