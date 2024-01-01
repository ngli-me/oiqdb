use sqlx::{migrate::MigrateDatabase, Sqlite};

//impl Image {
//    id: iqdb,
//    post_id: postId,
//    avglf1: f64,
//    avglf2: f64,
//    avglf3: f64,
//    sig: Vec<char>,
//
//}

const DB_URL: &str = "sqlite://sqlite.db";

pub async fn init_storage() {
    if !Sqlite::database_exists(DB_URL).await.unwrap_or(false) {
        println!("Creating database {}", DB_URL);
        match Sqlite::create_database(DB_URL).await {
            Ok(_) => println!("Create db success"),
            Err(error) => panic!("error: {}", error),
        }
    } else {
        println!("Database already exists");
    }

    //sqlx::query("CREATE TABLE users (name TEXT, age INTEGER)").execute(&pool).await?;

    //let 
}
