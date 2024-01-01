use image::GenericImageView;
use image::imageops::FilterType;

mod signature;
mod sqlite_db;

const NUM_PIXELS: usize = 128;

#[tokio::main]
async fn main() {
    let img = image::open("fxas9ze1gw3c1.jpg").unwrap();
    signature::haarsignature_from_file(img).await;
    sqlite_db::init_storage().await;
}

async fn from_file_content() {
    let img = image::open("fxas9ze1gw3c1.jpg").unwrap();

    println!("{:?}", img.color());
    let scaled = img.resize_exact(128, 128, FilterType::Triangle);
    println!("dimensions: {:?}", scaled.dimensions());
    let mut rchan: Vec<u8> = Vec::with_capacity(NUM_PIXELS);
    let mut gchan: Vec<u8> = Vec::with_capacity(NUM_PIXELS);
    let mut bchan: Vec<u8> = Vec::with_capacity(NUM_PIXELS);

    for p in scaled.pixels() {
        // The iteration order is x = 0 to width then y = 0 to height
        rchan.push(p.2[0]);
        gchan.push(p.2[1]);
        bchan.push(p.2[2]);
        //println!("{:?}", p)
    }

    let mut cdata1: Vec<()> = Vec::with_capacity(NUM_PIXELS * NUM_PIXELS);
    let mut cdata2: Vec<()> = Vec::with_capacity(NUM_PIXELS * NUM_PIXELS);
    let mut cdata3: Vec<()> = Vec::with_capacity(NUM_PIXELS * NUM_PIXELS);

}
