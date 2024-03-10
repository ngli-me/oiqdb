use anyhow::Result;
use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool};
use std::env;
use dotenvy::dotenv;

use crate::signature;

//impl Image {
//    id: iqdb,
//    post_id: postId,
//    avglf1: f64,
//    avglf2: f64,
//    avglf3: f64,
//    sig: Vec<char>,
//}
pub struct Sql {
    pool: SqlitePool,
}

impl Sql {
    pub async fn insert_image(pool: &SqlitePool, signature: signature::HaarSignature)
        -> Result<signature::HaarSignature> {
        //.into_iter().collect::<BitVec>().as_raw_slice();
        let mut conn = pool.acquire().await?;
        let id = sqlx::query(
        r#"
        INSERT INTO images ( id, avglf1, avglf2, avglf3, sig )
        VALUES ( NULL, ($1), ($2), ($3), ($4) )
        "#)
            .bind(signature.avglf[0])
            .bind(signature.avglf[1])
            .bind(signature.avglf[2])
            .bind(0x00)
            .execute(&mut *conn)
            .await?;

        Ok(signature)
    }
}

pub async fn run_db() -> Sql {
    Sql {
        pool: initialize_and_connect_storage()
            .await
            .expect("Error while initializing and connecting to database"),
    }

}

async fn initialize_and_connect_storage() -> Result<SqlitePool> {
    // Initialize the environment variables with dotenv
    dotenv().expect("Environment variable dotfile not found by dotenv!");

    // Set up the Sqlite database
    let database_url = &env::var("DATABASE_URL")?;
    if !Sqlite::database_exists(database_url).await.unwrap_or(false) {
        println!("Creating database {}", database_url);
        match Sqlite::create_database(database_url).await {
            Ok(_) => println!("Create db success"),
            Err(error) => panic!("error: {}", error),
        }
    } else {
        println!("Database already exists");
    }

    let conn = SqlitePool::connect(database_url).await?;

    println!("Running migrations");
    sqlx::migrate!().run(&conn).await?;

    Ok(conn)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_db_url() {
        dotenv().expect(".env file not found");
        assert!(!env::var("DATABASE_URL").expect("Error while getting env").is_empty());
    }
}
