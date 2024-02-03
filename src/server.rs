use axum::{
    extract::Multipart,
    http::StatusCode,
    response::{Response, IntoResponse},
    routing::{get, post},
    Json,
};
use axum_extra::{
    TypedHeader,
    headers::Origin,
};
use image::io::Reader;
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use tokio::{signal, task};

use crate::signature;

// here we show a type that implements Serialize + Send
#[derive(Serialize)]
pub struct Message {
    message: String,
}

enum ApiResponse {
    Ok,
    Created,
    JsonData(Vec<Message>),
}

pub async fn run() {
    // build our application by creating our router.
    let app = axum::Router::new()
        .fallback(fallback)
        .route("/", get(hello))
        .route("/users", post(create_user))
        .route("/image", post(images))
        .route("/upload", post(upload));

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

async fn create_user() -> (StatusCode, Json<User>) {
    // this will be converted into a JSON response
    // with a status code of `201 Created`
    println!("called create_user");
    (
        StatusCode::CREATED,
        Json(User {
            id: 10,
            username: "asd".to_string(),
        }),
    )
}

async fn upload(mut multipart: Multipart) -> Json<Message> {
    while let Some(mut field) = multipart.next_field().await.unwrap() {
        //let name = field.name().unwrap().to_string();
        let raw_data = field.bytes().await.unwrap();
        //let data_leng = raw_data.len();

        let img = Reader::new(Cursor::new(raw_data))
            .with_guessed_format()
            .expect("Error while unwrapping image");
        //ret = format!("Length of `{}` is {} bytes, with format {:?}", name, data_leng, img.format());
        let img = img.decode().unwrap();
        let _s = task::spawn_blocking(move || signature::HaarSignature::from(img)).await;
    }

    // Error response message -- multipart did not receive a file
    Json(Message {
        message: String::from("Ran through the upload function"),
    })
}

// the input to our `create_user` handler
#[derive(Deserialize)]
struct CreateUser {
    username: String,
}

// the output to our `create_user` handler
#[derive(Serialize)]
struct User {
    id: u64,
    username: String,
}

pub async fn query() {}
