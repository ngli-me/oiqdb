use image::imageops::FilterType;
use image::DynamicImage;
use serde::Serialize;

mod haar;

#[derive(Default, Serialize)]
pub struct HaarSignature {
    avglf: haar::LuminT,
    sig: haar::SignatureT,
}

impl HaarSignature {
    fn new() -> Self {
        Self {
            avglf: [0.0; haar::NUM_CHANNELS],
            sig: Default::default(),
        }
    }

    fn is_grayscale(&self) -> bool {
        self.avglf[1].abs() + self.avglf[2].abs() < (6.0 / 1000.0)
    }

    fn num_colors(&self) -> i32 {
        if self.is_grayscale() {
            1
        } else {
            3
        }
    }
}

impl From<DynamicImage> for HaarSignature {
    #[inline]
    fn from(filecontent: DynamicImage) -> Self {
        let filecontent = filecontent.resize_exact(128, 128, FilterType::Triangle);
        // Resize image and conver to YIQ
        let (a, b, c) = haar::transform_char(filecontent);
        let (avglf, sig): (haar::LuminT, haar::SignatureT) = haar::calc_haar(a, b, c);
        HaarSignature { avglf, sig }
    }
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
