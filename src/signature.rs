use image::imageops::FilterType;
use image::DynamicImage;
use serde::Serialize;
use std::ops::Index;

pub mod haar;

pub enum SigIndex {
    S0,
    S1,
    S2,
}

impl From<usize> for SigIndex {
    fn from(value: usize) -> Self {
        match value {
            0 => SigIndex::S0,
            1 => SigIndex::S1,
            2 => SigIndex::S2,
            _ => panic!("Index out of range for SigIndex"),
        }
    }
}

#[derive(Debug, Default, Serialize)]
pub struct HaarSignature {
    pub avglf: haar::Lumin,
    pub sig0: haar::SigT,
    pub sig1: haar::SigT,
    pub sig2: haar::SigT,
}

impl HaarSignature {
    pub fn new() -> Self {
        Self {
            avglf: [0.0; haar::N_COLORS],
            sig0: Default::default(),
            sig1: Default::default(),
            sig2: Default::default(),
        }
    }

    pub fn is_grayscale(&self) -> bool {
        self.avglf[1].abs() + self.avglf[2].abs() < (6.0 / 1000.0)
    }

    pub fn num_colors(&self) -> usize {
        if self.is_grayscale() {
            1
        } else {
            3
        }
    }
}

impl Index<SigIndex> for HaarSignature {
    type Output = [i16; haar::NUM_COEFS];

    fn index(&self, index: SigIndex) -> &Self::Output {
        match index {
            SigIndex::S0 => &self.sig0.sig,
            SigIndex::S1 => &self.sig1.sig,
            SigIndex::S2 => &self.sig2.sig,
        }
    }
}

impl Index<usize> for HaarSignature {
    type Output = [i16; haar::NUM_COEFS];

    fn index(&self, index: usize) -> &Self::Output {
        match SigIndex::from(index) {
            SigIndex::S0 => &self.sig0.sig,
            SigIndex::S1 => &self.sig1.sig,
            SigIndex::S2 => &self.sig2.sig,
        }
    }
}

impl From<DynamicImage> for HaarSignature {
    #[inline]
    fn from(filecontent: DynamicImage) -> Self {
        let filecontent = resize_image(filecontent);
        // Resize image and conver to YIQ
        let (a, b, c) = haar::transform_char(filecontent);
        let (avglf, sig0, sig1, sig2): (haar::Lumin, haar::SigT, haar::SigT, haar::SigT) =
            haar::calc_haar(a, b, c);
        HaarSignature {
            avglf,
            sig0,
            sig1,
            sig2,
        }
    }
}

fn resize_image(img: DynamicImage) -> DynamicImage {
    img.resize_exact(128, 128, FilterType::Triangle)
}

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
