
mod signature;
mod sqlite_db;

//#[tokio::main]
fn main() {
    let img = image::open("peppers.jpg").unwrap();
    signature::haarsignature_from_file(img);

    //sqlite_db::init_storage();
}


