use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Record {
    pub id: String,
    pub data: String,
}

#[cfg(test)]
mod tests {
    use sqlx::{Executor, MySql, Pool};

    use super::*;

    #[sqlx::test]
    fn test_record(pool: Pool<MySql>) {
        sqlx::migrate!("../migrations")
            .run(&pool)
            .await
            .expect("Failed to run migrations");

        let record = Record {
            id: "test_id".to_string(),
            data: "test_data".to_string(),
        };

        // Insert
        {
            let result = pool
                .execute(sqlx::query!(
                    "INSERT INTO Record (id, data) VALUES (?, ?)",
                    record.id,
                    record.data
                ))
                .await
                .expect("Failed to insert record");

            assert_eq!(result.rows_affected(), 1);
        }

        // Select
        {
            let result = sqlx::query_as!(
                Record,
                "SELECT id, data FROM Record WHERE id = ?",
                record.id
            )
            .fetch_one(&pool)
            .await
            .expect("Failed to get record");

            assert_eq!(result, record);
        }
    }
}
