mod db;
mod server;
mod signature;

#[tokio::main]
async fn main() {
    db::run_db().await;
    // run our application as a hyper server on http://localhost:3000.
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, server::router())
        .with_graceful_shutdown(server::shutdown_signal())
        .await
        .unwrap();
}
