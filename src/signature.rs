use std::fmt;
//use hex::decode_to_slice;
use image::{DynamicImage};
use image::imageops::FilterType;

mod haar;

#[derive(Default)]
pub struct HaarSignature {
    avglf: haar::LuminT,
    sig:   haar::SignatureT,
}

impl HaarSignature {
    fn new() -> Self {
        Self {
            avglf: [0.0; 3],
            sig:   Default::default(),
        }
    }

    fn is_grayscale(&self) -> bool {
        self.avglf[1].abs() + self.avglf[2].abs() < (6.0 / 1000.0)
    }

    fn num_colors(&self) -> i32 {
        if self.is_grayscale() { 1 } else { 3 }
    }
}

trait From {
    async fn from(filecontent: DynamicImage) -> Self;
}

//impl From<String> for HaarSignature {
//    #[inline]
//    fn from(hash: String) -> Self {
//        //if hash.size() != 5 + 2*size_of
//
//        let mut haar: HaarSignature;
//        let mut s = [0i16; 3 * haar::NUM_COEFS];
//        let mut p: String = hash[5..].to_string();
//        //decode_to_slice(p, &mut s as &mut [i16]);
//        haar
//    }
//}

impl From for HaarSignature {
    #[inline]
    async fn from(filecontent: DynamicImage) -> Self {
        let filecontent = filecontent.resize_exact(128, 128, FilterType::Triangle);
        let (a, b, c) = haar::transform_char(filecontent).await;
        println!("avglf: {:?} {:?} {:?}", a[0], b[0], c[0]);
        //let (avg, si) = haar::calc_haar(a, b, c).await;
        //HaarSignature { avglf: avg, sig: si }
        HaarSignature::new()
    }
}

pub async fn haarsignature_from_file(filecontent: DynamicImage) -> HaarSignature {
    let filecontent = filecontent.resize_exact(128, 128, FilterType::Triangle);
    let (a, b, c) = haar::transform_char(filecontent).await;
    println!("avglf: {:?} {:?} {:?}", a[0], b[0], c[0]);
    //let (avg, si) = haar::calc_haar(a, b, c).await;
    //HaarSignature { avglf: avg, sig: si }
    HaarSignature::new()
}
//impl fmt::Display for HaarSignature {
//    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
//        let mut str = "";
//        for name in &self.names {
//            fmt.write_str(str)?;
//            fmt.write_str(name)?;
//            str = ", ";
//        }
//        Ok(())
//    }
//}

// to json
