use anyhow::Result;
use lazy_static::lazy_static;
use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool};
use std::path::PathBuf;
use std::{env, fmt, fs};

//impl Image {
//    id: iqdb,
//    post_id: postId,
//    avglf1: f64,
//    avglf2: f64,
//    avglf3: f64,
//    sig: Vec<char>,
//
//}

pub async fn run_db() {
    initialize_and_connect_storage("test.db").await;
}


async fn initialize_and_connect_storage(database_url: &str) -> Result<SqlitePool> {
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

    sqlx::migrate!().run(&conn).await?;

    Ok(conn)
}


#[cfg(test)]
mod tests {
    use super::*;
}
