mod server;
mod signature;
mod sqlite_db;

/*
fn main() {
    let img = image::open("files/peppers.jpg").unwrap();
    let s = signature::HaarSignature::from(img);

}
*/

#[tokio::main]
pub async fn main() {
    // run our application as a hyper server on http://localhost:3000.
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, server::router())
        .with_graceful_shutdown(server::shutdown_signal())
        .await
        .unwrap();
}
