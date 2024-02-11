use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Record {
    pub id: String,
    pub data_cid: String,
}

#[cfg(test)]
mod tests {
    use sqlx::{Executor, MySqlPool};

    use super::*;

    #[sqlx::test]
    fn test_record(pool: MySqlPool) {
        let record = Record {
            id: "test_id".to_string(),
            data_cid: "test_data_cid".to_string(),
        };

        // Insert
        {
            let mut tx = pool.begin().await.expect("Failed to begin transaction");

            tx.execute(sqlx::query!(
                "INSERT INTO CidData (cid, path) VALUES (?, ?)",
                record.data_cid,
                "test_path"
            ))
            .await
            .expect("Failed to insert cid data");

            tx.execute(sqlx::query!(
                "INSERT INTO Record (id, data_cid) VALUES (?, ?)",
                record.id,
                record.data_cid
            ))
            .await
            .expect("Failed to insert record");

            tx.commit().await.expect("Failed to commit transaction");
        }

        // Select
        {
            let result = sqlx::query_as!(
                Record,
                "SELECT id, data_cid FROM Record WHERE id = ?",
                record.id
            )
            .fetch_one(&pool)
            .await
            .expect("Failed to get record");

            assert_eq!(result, record);
        }
    }
}
