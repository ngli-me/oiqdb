use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json,
};
use image::{io::Reader, DynamicImage};
use std::io::{Cursor, Error, ErrorKind};
use tokio::{signal, task};

use crate::iqdb::{db, IQDB};
use crate::{iqdb, signature};
use crate::signature::HaarSignature;

pub async fn router() -> axum::Router {
    let iqdb = iqdb::IQDB::new().await;
    axum::Router::new()
        .fallback(fallback)
        .route("/", get(hello))
        .route("/image", post(images))
        .route("/upload", post(query_image))
        .with_state(iqdb)
}

pub async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
        let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

/// axum handler for any request that fails to match the router routes.
/// this implementation returns http status code not found (404).
async fn fallback(uri: axum::http::Uri) -> impl IntoResponse {
    (
        axum::http::StatusCode::NOT_FOUND,
        format!("no route {}", uri),
    )
}

/// axum handler for "get /" which returns a string and causes axum to
/// immediately respond with status code `200 ok` and with the string.
async fn hello() -> &'static str {
    "hello, world!"
}

async fn images(State(iqdb): State<IQDB>) -> (StatusCode, &'static str) {
    (StatusCode::OK, "called images")
}

// Axum Route for ...
async fn upload(State(sql): State<db::Sql>) {}

// Handler
async fn query_image(State(iqdb): State<IQDB>, multipart: Multipart) -> Response {
    let res = match extract_image(multipart).await {
        Ok(img) => img,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };
    // Calculate the Haar Signature
    let sig: HaarSignature = task::spawn_blocking(move || signature::HaarSignature::from(res))
        .await
        .expect("Error while generating haar signature");

    // Insert into the db
    //sql.insert_signature(&sig).await;

    // Give back the requester something to chew on
    Json(sig).into_response()
}

async fn extract_image(mut multipart: Multipart) -> Result<DynamicImage, Error> {
    while let Some(field) = multipart.next_field().await.unwrap() {
        //let name = field.name().unwrap().to_string();
        let raw_data = field.bytes().await.unwrap();
        let read_image = Reader::new(Cursor::new(raw_data))
            .with_guessed_format()
            .expect("Error while unwrapping image.");
        //ret = format!("Length of `{}` is {} bytes, with format {:?}", name, data_leng, img.format());
        return Ok(read_image
            .decode()
            .expect("Error while decoding the image."));
    }
    Err(Error::new(ErrorKind::InvalidInput, "No input found"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum_test::TestServer;

    #[tokio::test]
    async fn route_tests() {
        let server = TestServer::new(router().await).unwrap();

        let response = server.get(&"/").await;

        response.assert_status_ok();
        response.assert_text("hello, world!");
    }
}
