use crate::iqdb::imgdb::{ImgBin, ImgBinState};
use std::sync::Arc;
use futures::TryStreamExt;
use sqlx::Row;
use tokio::sync::Mutex;

mod db;
mod imgdb;

#[derive(Clone)]
pub struct IQDB {
    pub state: ImgBinState,
    pub sql: db::Sql,
}

impl IQDB {
    pub async fn new() -> sqlx::Result<Self, sqlx::Error> {
        let sql = db::Sql::new().await;
        let state = ImgBinState {
            data: Arc::new(Mutex::new(ImgBin::new())),
        };

        // I think this should probably just be an instance of pool and not need cloning
        let sql_clone: db::Sql = sql.clone();
        let mut sql_rows = sql_clone.each_image();
        while let Some(r) = sql_rows.try_next().await? {
            println!("the sqlite row was gotten: {}", r.id);
        }
        /*
        sqlite_db_->eachImage([&](const auto& image) {
        addImageInMemory(image.id, image.post_id, image.haar());
        if (image.id % 250000 == 0) {
          INFO("Loaded image {} (post #{})...\n", image.id, image.post_id);
        }
        });
        INFO("Loaded {} images from {}.\n", getImgCount(), filename);
        */

        Ok(IQDB {
            state: state,
            sql: sql,
        })
    }
}
