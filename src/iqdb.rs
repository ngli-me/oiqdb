use std::sync::Arc;
use tokio::sync::Mutex;
use crate::iqdb::imgdb::{ImgBin, ImgBinState};

pub mod db;
mod imgdb;

#[derive(Clone)]
pub struct IQDB {
    state: ImgBinState,
    sql: db::Sql,
}

impl IQDB {
    pub async fn new() -> Self {
        let sql = db::run_db().await;
        let state = ImgBinState { data: Arc::new(Mutex::new(ImgBin::new())) };

        /*
        sqlite_db_->eachImage([&](const auto& image) {
        addImageInMemory(image.id, image.post_id, image.haar());
        if (image.id % 250000 == 0) {
          INFO("Loaded image {} (post #{})...\n", image.id, image.post_id);
        }
        });
        INFO("Loaded {} images from {}.\n", getImgCount(), filename);
        */

        IQDB {
            state: state,
            sql: sql,
        }
    }
}