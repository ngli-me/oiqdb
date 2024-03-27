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

use crate::db::{self, Sql};
use crate::signature;

pub async fn router() -> axum::Router {
    let sql = db::run_db().await;
    axum::Router::new()
        .fallback(fallback)
        .route("/", get(hello))
        .route("/image", post(images))
        .route("/upload", post(query_image))
        .with_state(sql)
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
async fn fallback(uri: axum::http::Uri) -> impl axum::response::IntoResponse {
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

async fn images(State(sql): State<Sql>) -> (StatusCode, &'static str) {
    (StatusCode::OK, "called images")
}

// Axum Route for ...
async fn upload(State(sql): State<Sql>) {}

// Handler
async fn query_image(State(sql): State<Sql>, multipart: Multipart) -> Response {
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
    use hyper_util::client::legacy::{connect, Client};

    #[tokio::test]
    async fn route_tests() {
        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            axum::serve(listener, router()).await.unwrap();
        });
        let client = Client::builder(hyper_util::rt::TokioExecutor::new()).build_http();

        async fn check_response(
            client: Client<connect::HttpConnector, axum_core::body::Body>,
            addr: std::net::SocketAddr,
            path: String,
            body: axum::body::Body,
            status: StatusCode,
        ) {
            let response = client
                .request(
                    axum::http::Request::builder()
                        .uri(format!("http://{addr}{path}"))
                        .header("Host", "localhost")
                        .body(body)
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(response.status(), status)
        }

        check_response(
            client,
            addr,
            "".to_string(),
            axum::body::Body::empty(),
            StatusCode::OK,
        )
        .await;
    }
}
