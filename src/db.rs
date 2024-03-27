use anyhow::Result;
use dotenvy::dotenv;
use sqlx::sqlite::SqliteQueryResult;
use sqlx::SqlitePool;
use std::env;

use crate::signature;
use crate::signature::haar::ToBits;

#[derive(Clone)]
pub struct Sql {
    pool: SqlitePool,
}

impl Sql {
    pub async fn insert_signature(
        &self,
        id: u32,
        mut signature: signature::HaarSignature,
    ) -> Result<i64> {
        let mut conn = self.pool.acquire().await?;
        let blob = signature.sig.get_blob();
        let id = sqlx::query!(
            r#"
        INSERT INTO images ( id, avglf1, avglf2, avglf3, sig )
        VALUES ( ($1), ($2), ($3), ($4), ($5) )
        "#,
            id,
            signature.avglf[0],
            signature.avglf[1],
            signature.avglf[2],
            blob
        )
        .execute(&mut *conn)
        .await?
        .last_insert_rowid();

        Ok(id)
    }

    pub async fn list_rows(&self) -> Result<()> {
        let rows = sqlx::query!(
            r#"
            SELECT id, avglf1, avglf2, avglf3
            FROM images
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        for r in rows {
            println!("- [{}]: {} {} {}", r.id, &r.avglf1, &r.avglf2, &r.avglf3,);
        }

        Ok(())
    }

    pub async fn remove_image(&self, mut id: u32) -> Result<SqliteQueryResult> {
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

pub async fn run_db() -> Sql {
    Sql {
        pool: initialize_and_connect_storage(get_db_url().await.unwrap().as_str())
            .await
            .expect("Error while initializing and connecting to database"),
    }
}

async fn get_db_url() -> Result<String> {
    dotenv().expect("Environment variable dotfile not found by dotenv.");
    Ok(env::var("DATABASE_URL")?)
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
    use crate::signature::haar::SignatureT;
    use regex::Regex;
    use std::fs;
    use std::path::Path;

    static TMP_FILES: [&str; 2] = ["shm", "wal"];

    #[tokio::test]
    #[doc = include_str!("../doc/db/test.md")]
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
        let s: haar::SigT = [0; haar::NUM_COEFS];
        let sig = signature::HaarSignature {
            // Create a blank haar signature to insert
            avglf: [0.0, 0.0, 0.0],
            sig: SignatureT { sig: [s, s, s] },
        };

        let id: u32 = 555;

        let _ = sql
            .insert_signature(id, sig)
            .await
            .expect("Error while inserting signature.");
        println!("Added new entry with id {id}.");

        let _ = sql.list_rows().await.expect("Error while listing rows");

        // Remove image
        let _ = sql
            .remove_image(id)
            .await
            .expect("Error while removing id: {id}");
        println!("Running remove image");

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
