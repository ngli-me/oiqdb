use dotenvy::dotenv;
use futures::stream::BoxStream;
use futures::{FutureExt, Stream, TryStreamExt};
use sqlx::sqlite::{SqliteQueryResult, SqliteRow};
use sqlx::{Error, FromRow, Row, SqlitePool};
use std::env;
use std::env::VarError;

use crate::signature;
use crate::signature::HaarSignature;

#[derive(Clone)]
pub struct Sql {
    pool: SqlitePool,
}

pub struct SqlRow {
    pub id: u32,
    pub s: HaarSignature,
}

impl FromRow<'_, SqliteRow> for SqlRow {
    fn from_row(row: &'_ SqliteRow) -> sqlx::Result<Self, Error> {
        Ok(Self {
            id: row.try_get("id").unwrap(),
            s: HaarSignature {
                avglf: [
                    row.try_get("avglf0")?,
                    row.try_get("avglf1")?,
                    row.try_get("avglf2")?,
                ],
                sig0: serde_json::from_slice(row.try_get("sig0")?).unwrap(), // TODO: dont like the use of unwrap here
                sig1: serde_json::from_slice(row.try_get("sig1")?).unwrap(),
                sig2: serde_json::from_slice(row.try_get("sig2")?).unwrap(),
            },
        })
    }
}

impl Sql {
    pub async fn new() -> Self {
        match get_db_url().await {
            Ok(url) => Sql {
                pool: initialize_and_connect_storage(url.as_str())
                    .await
                    .expect("Error while initializing and connecting to database"),
            },
            Err(err) => panic!("DB environment variable issue, {}", err),
        }
    }

    pub async fn insert_signature(&self, signature: &HaarSignature) -> Option<i64> {
        match self.pool.acquire().await {
            Ok(mut conn) => {
                let blob0 = serde_json::to_vec(&signature.sig0).unwrap();
                let blob1 = serde_json::to_vec(&signature.sig1).unwrap();
                let blob2 = serde_json::to_vec(&signature.sig2).unwrap();

                sqlx::query!(
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
                .await
                .map_or(None, |query_result| Some(query_result.last_insert_rowid()))
            }
            Err(_) => None,
        }
    }

    pub async fn get_image(&self, id: u32) -> Option<SqlRow> {
        sqlx::query_as(
            r#"
            SELECT id, avglf0, avglf1, avglf2, sig0, sig1, sig2
            FROM images
            WHERE id = (?)
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .unwrap_or(None)
    }

    pub async fn list_rows(&self) -> Option<i64> {
        sqlx::query!(
            r#"
            SELECT id, avglf0, avglf1, avglf2, sig0, sig1, sig2
            FROM images
            ORDER BY id ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_or(None, |rows| {
            for r in rows {
                println!("- [{}]: {} {} {}", r.id, &r.avglf0, &r.avglf1, &r.avglf2,);
            }
            Some(1)
        })
    }

    pub fn each_image(&self) -> BoxStream<sqlx::Result<SqlRow>> {
        sqlx::query_as(
            r#"
            SELECT id, avglf0, avglf1, avglf2, sig0, sig1, sig2
            FROM images
            ORDER BY id ASC
            "#,
        )
        .fetch(&self.pool)
    }

    pub async fn remove_image(&self, id: u32) -> Result<SqliteQueryResult, Error> {
        let mut conn = self.pool.acquire().await?;
        sqlx::query!(
            r#"
            DELETE FROM images
            WHERE id = ($1)
            "#,
            id
        )
        .execute(&mut *conn)
        .await
    }
}

async fn get_db_url() -> Result<String, VarError> {
    dotenv().expect("Environment variable dotfile not found by dotenv.");
    env::var("DATABASE_URL")
}

async fn initialize_and_connect_storage(url: &str) -> Option<SqlitePool> {
    match SqlitePool::connect(url).await {
        Ok(pool) => run_migrations(pool).await,
        Err(_) => None,
    }
}

async fn run_migrations(pool: SqlitePool) -> Option<SqlitePool> {
    match sqlx::migrate!().run(&pool).await {
        Ok(_) => Some(pool),
        Err(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signature::haar;
    use regex::Regex;
    use std::path::Path;

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
        let re = Regex::new(r"(?:sqlite://)?(.+)").unwrap();
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
            sig0: haar::SigT {
                sig: [0; haar::NUM_COEFS],
            },
            sig1: haar::SigT {
                sig: [0; haar::NUM_COEFS],
            },
            sig2: haar::SigT {
                sig: [0; haar::NUM_COEFS],
            },
        };

        let id = sql
            .insert_signature(&sig)
            .await
            .expect("Error while inserting signature.");
        println!("Added new entry with id {id}.");
        let _ = sql.list_rows().await.expect("Error while listing rows");

        let img = sql.get_image(id as u32).await.unwrap();
        println!("For id: {id}, the SqlRow's HaarSignature is: {:?}", img.s);

        // Remove image
        println!("Running remove image for id: {id}");
        let _ = sql
            .remove_image(id as u32)
            .await
            .expect("Error while removing id: {id}");
        let _ = sql.list_rows().await.expect("Error while listing rows");

        sql.pool.close().await;
    }
}
