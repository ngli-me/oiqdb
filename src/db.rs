use anyhow::Result;
use dotenvy::dotenv;
use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool};
use std::env;

use crate::signature;
use crate::signature::haar::ToBits;

pub struct Sql {
    pool: SqlitePool,
}

impl Sql {
    pub async fn insert_signature(&self, mut signature: signature::HaarSignature) -> Result<i64> {
        let mut conn = self.pool.acquire().await?;
        let id = sqlx::query(
            r#"
        INSERT INTO images ( id, avglf1, avglf2, avglf3, sig )
        VALUES ( NULL, ($1), ($2), ($3), ($4) )
        "#,
        )
        .bind(signature.avglf[0])
        .bind(signature.avglf[1])
        .bind(signature.avglf[2])
        .bind(signature.sig.get_blob())
        .execute(&mut *conn)
        .await?
        .last_insert_rowid();

        Ok(id)
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
    // Set up the Sqlite database
    if !Sqlite::database_exists(url).await.unwrap_or(false) {
        println!("Database doesn't exist, creating database {}.", url);
        match Sqlite::create_database(url).await {
            Ok(_) => println!("Create db success."),
            Err(error) => panic!("Error while creating database: {}.", error),
        }
    }

    let conn = SqlitePool::connect(url).await?;

    println!("Running migrations");
    sqlx::migrate!().run(&conn).await?;

    Ok(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signature::haar;
    use crate::signature::haar::SignatureT;
    use std::fs;
    use std::path::Path;

    static DB: &str = "tmp.db";
    static TMP_FILES: [&str; 2] = ["shm", "wal"];

    #[tokio::test]
    async fn tmp() {
        let environment = "test/init.env";
        dotenvy::from_filename(environment).expect("Test env file not found.");
        let url = env::var("DATABASE_URL").unwrap();

        let sql = Sql {
            pool: initialize_and_connect_storage(url.as_str())
                .await
                .expect("Error while initializing and connecting to database."),
        };

        let db = Path::new(DB);
        assert!(db.exists());

        let s: haar::SigT = [0; haar::NUM_COEFS];
        // Test inserting an entry
        let sig = signature::HaarSignature {
            // Create a blank haar signature to insert
            avglf: [0.0, 0.0, 0.0],
            sig: SignatureT { sig: [s, s, s] },
        };

        let id = sql
            .insert_signature(sig)
            .await
            .expect("Error while inserting signature.");
        println!("Added new entry with id {id}.");

        // Cleanup tmp sqlite db files
        fs::remove_file(db).unwrap();
        for suffix in TMP_FILES {
            let name = format!("{DB}-{suffix}");
            let tmp = Path::new(name.as_str());
            if tmp.exists() {
                println!("Removing tmp file: {:?}", tmp);
                fs::remove_file(tmp).unwrap();
            }
        }
    }
}
