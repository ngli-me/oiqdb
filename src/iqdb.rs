use std::sync::Arc;
use tokio::sync::Mutex;
use crate::iqdb::imgdb::{ImgBin, ImgBinState};

mod db;
mod imgdb;

#[derive(Clone)]
pub struct IQDB {
    pub state: ImgBinState,
    pub sql: db::Sql,
}

impl IQDB {
    pub async fn new() -> Self {
        let sql = db::Sql::new().await;
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