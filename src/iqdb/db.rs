use anyhow::Result;
use dotenvy::dotenv;
use sqlx::sqlite::SqliteQueryResult;
use sqlx::SqlitePool;
use std::env;
use std::env::VarError;

use crate::signature;
use crate::signature::HaarSignature;

#[derive(Clone)]
pub struct Sql {
    pool: SqlitePool,
}

impl Sql {
    pub async fn new() -> Self {
        match get_db_url().await {
            Ok(url) => {
                Sql {
                    pool: initialize_and_connect_storage(url.as_str())
                        .await
                        .expect("Error while initializing and connecting to database"),
                }
            }
            Err(err) => panic!("DB environment variable issue, {}", err)
        }
    }

    pub async fn insert_signature(
        &self,
        signature: &signature::HaarSignature,
    ) -> Result<i64> {
        let mut conn = self.pool.acquire().await?;
        let blob0 = serde_json::to_vec(&signature.sig0).unwrap();
        let blob1 = serde_json::to_vec(&signature.sig1).unwrap();
        let blob2 = serde_json::to_vec(&signature.sig2).unwrap();
        let id = sqlx::query!(
            r#"
        INSERT INTO images ( avglf0, avglf1, avglf2, sig0, sig1, sig2 )
        VALUES ( ($1), ($2), ($3), ($4), ($5), ($6))
        "#,
            signature.avglf[0], // TODO: looks like some possible issues with this, REAL is f64
            signature.avglf[1],
            signature.avglf[2],
            blob0,
            blob1,
            blob2
        )
            .execute(&mut *conn)
            .await?
            .last_insert_rowid();

        Ok(id)
    }

    pub async fn get_one(&self) -> Result<HaarSignature> {
        Ok(
            sqlx::query_as(
                r#"
            SELECT avglf0, avglf1, avglf2, sig0, sig1, sig2
            FROM images
            "#,
            )
                .fetch_one(&self.pool)
                .await?
        )
    }

    pub async fn list_rows(&self) -> Result<()> {
        let rows = sqlx::query!(
            r#"
            SELECT id, avglf0, avglf1, avglf2, sig0, sig1, sig2
            FROM images
            "#,
        )
            .fetch_all(&self.pool)
            .await?;

        for r in rows {
            println!("- [{}]: {} {} {}", r.id, &r.avglf0, &r.avglf1, &r.avglf2, );
        }

        Ok(())
    }

    pub async fn remove_image(&self, mut id: i64) -> Result<SqliteQueryResult> {
        let mut conn = self.pool.acquire().await?;
        let res = sqlx::query!(
            r#"
            DELETE FROM images
            WHERE id = ($1)
            "#,
            id
        )
            .execute(&mut *conn)
            .await?;

        Ok(res)
    }
}

async fn get_db_url() -> Result<String, VarError> {
    dotenv().expect("Environment variable dotfile not found by dotenv.");
    env::var("DATABASE_URL")
}

async fn initialize_and_connect_storage(url: &str) -> Result<SqlitePool> {
    let conn = SqlitePool::connect(url).await?;
    sqlx::migrate!().run(&conn).await?;
    Ok(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signature::haar;
    use regex::Regex;
    use std::fs;
    use std::path::Path;

    static TMP_FILES: [&str; 2] = ["shm", "wal"];

    #[tokio::test]
    #[doc = include_str!("../../doc/db/test.md")]
    async fn test() {
        // Initialize the test db
        // Ensure the db file is created according to the env file
        dotenv().expect("Env file not found.");
        let url = env::var("DATABASE_URL").unwrap();

        let sql = Sql {
            pool: initialize_and_connect_storage(url.as_str())
                .await
                .expect("Error while initializing and connecting to database."),
        };

        // Get the filepath for the db from the environment variable
        let re = Regex::new(r"(?:sqlite:\/\/)?(.+)").unwrap();
        let Some(caps) = re.captures(&url) else {
            return;
        };
        assert!(!&caps[1].is_empty());

        let db = Path::new(&caps[1]);
        assert!(db.exists());

        // Insert a signature
        let sig = signature::HaarSignature {
            // Create a blank haar signature to insert
            avglf: [0.0, 0.0, 0.0],
            sig0: haar::SigT { sig: [0; haar::NUM_COEFS] },
            sig1: haar::SigT { sig: [0; haar::NUM_COEFS] },
            sig2: haar::SigT { sig: [0; haar::NUM_COEFS] },
        };

        let id = sql
            .insert_signature(&sig)
            .await
            .expect("Error while inserting signature.");
        println!("Added new entry with id {id}.");

        let _ = sql.get_one().await;

        let _ = sql.list_rows().await.expect("Error while listing rows");

        // Remove image
        let _ = sql
            .remove_image(id)
            .await
            .expect("Error while removing id: {id}");
        println!("Running remove image for id: {id}");

        let _ = sql.list_rows().await.expect("Error while listing rows");

        sql.pool.close().await;

        // Cleanup tmp sqlite db files
        for suffix in TMP_FILES {
            let name = format!("{url}-{suffix}");
            let tmp = Path::new(name.as_str());
            if tmp.exists() {
                println!("Removing tmp file: {:?}", tmp);
                fs::remove_file(tmp).unwrap();
            }
        }
    }
}
