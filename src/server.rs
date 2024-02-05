use axum::{
    extract::Multipart,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json,
};
use image::{io::Reader, DynamicImage};
use serde::{Deserialize, Serialize};
use std::io::{Cursor, Error, ErrorKind};
use tokio::{signal, task};

use crate::signature;

// here we show a type that implements Serialize + Send
#[derive(Deserialize, Serialize)]
pub struct Message {
    message: String,
}

pub async fn run() {
    // build our application by creating our router.
    let app = axum::Router::new()
        .fallback(fallback)
        .route("/", get(hello))
        .route("/image", post(images))
        .route("/upload", post(query_image));

    // run our application as a hyper server on http://localhost:3000.
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("{:?}", 0);
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn shutdown_signal() {
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
pub async fn fallback(uri: axum::http::Uri) -> impl axum::response::IntoResponse {
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

async fn images() -> (StatusCode, &'static str) {
    (StatusCode::OK, "called images")
}

// Axum Route for ...
async fn upload() {}

// Handler
async fn query_image(mut multipart: Multipart) -> Response {
    let res = match extract_image(multipart).await {
        Ok(img) => img,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    Json(
        // Calculate the Haar Signature
        task::spawn_blocking(move || signature::HaarSignature::from(res))
            .await
            .expect("Error while generating haar signature"),
    )
    .into_response()
}

async fn extract_image(mut multipart: Multipart) -> Result<DynamicImage, Error> {
    while let Some(mut field) = multipart.next_field().await.unwrap() {
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
