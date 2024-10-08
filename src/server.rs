use axum::{
    extract::{Multipart, State},
    http::{StatusCode, Uri},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json,
};
use image::{DynamicImage, ImageReader};
use std::io::{Cursor, Error, ErrorKind};
use tokio::{signal, task};

use crate::signature::HaarSignature;
use crate::{iqdb::IQDB, signature};

pub async fn router() -> axum::Router {
    let iqdb = IQDB::new().await;
    axum::Router::new()
        .fallback(fallback)
        .route("/", get(hello))
        .route("/upload", post(query_image))
        .with_state(iqdb.unwrap())
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
async fn fallback(uri: Uri) -> impl IntoResponse {
    (StatusCode::NOT_FOUND, format!("no route {}", uri))
}

/// axum handler for "get /" which returns a string and causes axum to
/// immediately respond with status code `200 ok` and with the string.
async fn hello() -> &'static str {
    "hello, world!"
}

// Axum Route for ...
async fn upload(State(iqdb): State<IQDB>) {}

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
    let id = iqdb.sql.insert_signature(&sig).await;

    Json(sig).into_response()
}

async fn extract_image(mut multipart: Multipart) -> Result<DynamicImage, Error> {
    while let Some(field) = multipart.next_field().await.unwrap() {
        let raw_data = field.bytes().await.unwrap();
        let read_image = ImageReader::new(Cursor::new(raw_data))
            .with_guessed_format()
            .expect("Error while unwrapping image.");
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
