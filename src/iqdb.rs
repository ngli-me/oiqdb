use crate::iqdb::imgdb::{ImgBin, ImgBinState};
use futures::TryStreamExt;
use sqlx::Row;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::signature::HaarSignature;

mod db;
mod imgdb;

#[derive(Clone)]
pub struct IQDB {
    pub state: ImgBinState,
    pub sql: db::Sql,
}

impl IQDB {
    // this is load database lel
    pub async fn new() -> sqlx::Result<Self, sqlx::Error> {
        let sql = db::Sql::new().await;
        let state = ImgBinState {
            data: Arc::new(Mutex::new(ImgBin::new())),
        };

        let sql_clone: db::Sql = sql.clone();
        let mut sql_rows = sql_clone.each_image();
        while let Some(r) = sql_rows.try_next().await? {
            println!("the sqlite row was gotten: {}", r.id);
            state.data.clone().lock().await.add_image_in_memory(r.id, r.id, &r.s);
            if r.id % 250000 == 0 {
                println!("loaded a bunch of images");
            }
        }

        Ok(IQDB {
            state: state,
            sql: sql,
        })
    }

    pub async fn add_image(&self, haar: &HaarSignature) -> Option<u32> {
        match self.sql.insert_signature(haar).await {
            Some(id) => {
                self.state
                    .data
                    .clone()
                    .lock()
                    .await.add_image_in_memory(id as imgdb::IqdbId, id as imgdb::PostId, haar)
            }
            None => None,
        }
    }

    pub async fn remove_image(&self, post_id: imgdb::PostId) -> Option<imgdb::PostId> {
        let image = self.sql.get_image(post_id).await;
        if image.is_none() {
            // add some logging ig
            return None;
        }
        let image = image.unwrap();
        self.state
            .data
            .clone()
            .lock()
            .await
            .remove_image(&image.s, image.id);
        self.sql.remove_image(post_id).await.map_or(None, |_| Some(post_id))
    }
}
