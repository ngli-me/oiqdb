use image::imageops::FilterType;
use image::DynamicImage;
use serde::Serialize;
use sqlx::FromRow;

pub mod haar;

// Weights for the Haar coefficients.
// Straight from the referenced paper:
const WEIGHTS: [&[f32; 3]; 6] = [
    // For scanned picture (sketch=0):
    //    Y      I      Q       idx total occurs
    &[5.00, 19.21, 34.37],   // 0   58.58      1 (`DC' component)
    &[0.83, 01.26, 00.36],   // 1    2.45      3
    &[1.01, 00.44, 00.45],   // 2    1.90      5
    &[0.52, 00.53, 00.14],   // 3    1.19      7
    &[0.47, 00.28, 00.18],   // 4    0.93      9
    &[0.30, 00.14, 00.27]];  // 5    0.71      16384-25=16359

#[derive(FromRow)]
#[derive(Default, Serialize)]
pub struct HaarSignature {
    pub avglf: haar::LuminT,
    pub sig: haar::SignatureT,
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
        let filecontent = resize_image(filecontent);
        // Resize image and conver to YIQ
        let (a, b, c) = haar::transform_char(filecontent);
        let (avglf, sig): (haar::LuminT, haar::SignatureT) = haar::calc_haar(a, b, c);
        HaarSignature { avglf, sig }
    }
}

fn resize_image(img: DynamicImage) -> DynamicImage {
    img.resize_exact(128, 128, FilterType::Triangle)
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

#[cfg(test)]
mod test {
    use super::*;
    use std::fs::File;
    use std::io::{self, BufRead};
    use std::path::Path;

    const PATH: &str = "reference/";
    const RESIZE: [&str; 3] = ["r_resize.txt", "g_resize.txt", "b_resize.txt"];

    #[test]
    fn testreference() {
        for channel in RESIZE {
            let img = read_i32_vector_file((PATH.to_owned() + channel).to_string());
            assert_eq!(img.len(), haar::NUM_PIXELS_SQUARED - 1);
        }
    }

    fn read_i32_vector_file(filename: String) -> Vec<i32> {
        let mut ret: Vec<i32> = vec![];
        if let Ok(lines) = read_lines(filename) {
            // Consumes the iterator, returns an (Optional) String
            // For these I'm using only 1 line
            for line in lines.flatten() {
                let iter = line[4..(line.len() - 1)].split(",");

                for value in iter {
                    match value.parse::<i32>() {
                        Ok(n) => ret.push(n),
                        Err(e) => println!("Parsing error for : {}, with error {}", value, e),
                    }
                }
            }
        }
        ret
    }

    // The output is wrapped in a Result to allow matching on errors.
    // Returns an Iterator to the Reader of the lines of the file.
    fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
        where
            P: AsRef<Path>,
    {
        let file = File::open(filename)?;
        Ok(io::BufReader::new(file).lines())
    }
}
