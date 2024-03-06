use anyhow::Result;
use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool};
use std::env;
use std::mem::transmute;

use crate::signature::HaarSignature;

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
    initialize_and_connect_storage().await;
}

async fn initialize_and_connect_storage() -> Result<SqlitePool> {
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

    sqlx::migrate!().run(&conn).await?;

    Ok(conn)
}

async fn insert_image(pool: &SqlitePool, signature: HaarSignature) -> Result<HaarSignature> {
    let sig_bytes = unsafe {
        transmute::<SignatureT, &[char]>(signature.sig)
    };
    //.into_iter().collect::<BitVec>().as_raw_slice();
    let mut conn = pool.acquire().await?;
    let id = sqlx::query!(
        r#"
        INSERT INTO images ( id, avglf1, avglf2, avglf3, sig )
        VALUES ( NULL, ?1, ?2, ?3, ?4 )
        "#,
        signature.avglf[0],
        signature.avglf[1],
        signature.avglf[2],
        sig_bytes
    )
    .execute(&mut *conn)
    .await?
    .last_insert_rowid();

    Ok(id)
}

#[cfg(test)]
mod tests {
    use super::*;
}
