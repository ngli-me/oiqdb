use crate::iqdb::imgdb::{ImgBin, ImgBinState};
use futures::TryStreamExt;
use sqlx::Row;
use std::sync::Arc;
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

    /*pub async fn add_image(&self, post_id: imgdb::ImageId) {
        self.remove_image()
    }*/

    pub async fn remove_image(&self, post_id: imgdb::PostId) -> Option<imgdb::PostId> {
        let image = self.sql.get_image(post_id).await;
        if image.is_none() {
            // add some logging ig
            return None;
        }
        let image = image.unwrap();
        // TODO: https://itsallaboutthebit.com/arc-mutex/ Might be able to use Mutex without Arc
        self.state
            .data
            .clone()
            .lock()
            .await
            .remove_image(&image.s, image.id);
        self.sql.remove_image(post_id).await;

        return Some(post_id);
    }
}
