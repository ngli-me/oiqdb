mod signature;
mod sqlite_db;

//#[tokio::main]
fn main() {
    let img = image::open("files/peppers.jpg").unwrap();
    signature::HaarSignature::from(img);

    //sqlite_db::init_storage();
}


