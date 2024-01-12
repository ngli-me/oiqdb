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
    server::run().await;
}
