use bitvec::macros::internal::funty::Fundamental;
use bitvec::prelude::{BitVec, Lsb0};
use bitvec::view::AsBits;
use image::imageops::FilterType;
use image::{DynamicImage, GenericImageView};
use itertools::izip;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::f32::consts::FRAC_1_SQRT_2;

pub const NUM_CHANNELS: usize = 3;
pub const NUM_COEFS: usize = 40;
pub const NUM_PIXELS: usize = 128;
pub const NUM_PIXELS_SQUARED: usize = NUM_PIXELS.pow(2);
pub const SCALING_FACTOR: f32 = 256.0 * 128.0;

pub type Idx = i16;
pub type LuminT = [f32; NUM_CHANNELS];
pub type SigT = [i16; NUM_COEFS];

#[serde_as]
#[derive(Deserialize, Serialize)]
pub struct SignatureT {
    #[serde_as(as = "[[_; NUM_COEFS]; 3]")]
    pub sig: [SigT; NUM_CHANNELS],
}

pub trait ToBits {
    fn flatten(&mut self) -> Vec<i16>;
    fn flatten_and_serialize(&mut self) -> BitVec<u16, Lsb0>;
    fn get_blob(&mut self) -> String;
}

impl ToBits for SignatureT {
    fn flatten(&mut self) -> Vec<i16> {
        self.sig
            .iter()
            .flat_map(|array| array.iter())
            .cloned()
            .collect::<Vec<i16>>()
    }

    fn flatten_and_serialize(&mut self) -> BitVec<u16, Lsb0> {
        let flat = self.flatten();
        let flat_arr = <&[i16; NUM_COEFS * NUM_CHANNELS]>::try_from(flat.as_slice()).unwrap();
        flat_arr.map(i16::as_u16).as_bits::<Lsb0>().to_bitvec()
    }

    fn get_blob(&mut self) -> String {
        let b = self.flatten_and_serialize();
        format!("{:x}", b).replace(&['[', ',', ' ', ']'], "")
    }
}

impl Default for SignatureT {
    fn default() -> Self {
        SignatureT {
            sig: [[0; NUM_COEFS]; NUM_CHANNELS],
        }
    }
}

pub fn transform_char(img: DynamicImage) -> (Vec<f32>, Vec<f32>, Vec<f32>) {
    let img = img.resize_exact(128, 128, FilterType::Triangle);
    let (mut a, mut b, mut c) = rgb_to_yiq_conversion(img);
    haar_2d(&mut a);
    haar_2d(&mut b);
    haar_2d(&mut c);

    // Reintroduce the skipped scaling factors
    a[0] /= SCALING_FACTOR;
    b[0] /= SCALING_FACTOR;
    c[0] /= SCALING_FACTOR;

    (a, b, c)
}

fn haar_rows(a: &mut Vec<f32>) {
    for i in (0..NUM_PIXELS_SQUARED).step_by(NUM_PIXELS) {
        let (mut h, mut h1): (usize, usize);
        let mut c: f32 = 1.0;
        h = NUM_PIXELS;
        while h > 1 {
            let (mut j1, mut j2, mut k): (usize, usize, usize) = (i, i, 0);

            h1 = h >> 1; // h1 = h / 2
            c = c * FRAC_1_SQRT_2;
            let mut t: Vec<f32> = vec![0.0; h1];
            while k < h1 {
                let j21: usize = j2 + 1;
                t[k] = (a[j2] - a[j21]) * c;
                a[j1] = a[j2] + a[j21];

                k += 1;
                j1 += 1;
                j2 += 2;
            }

            // Write back subtraction results:
            a.splice((i + h1)..(i + h), t);

            h = h1;
        }
        // Fix first element of each row:
        a[i] *= c;
    }
}

fn haar_columns(a: &mut Vec<f32>) {
    for i in 0..NUM_PIXELS {
        let (mut h, mut h1): (usize, usize);
        let mut c: f32 = 1.0;
        h = NUM_PIXELS;
        while h > 1 {
            let (mut j1, mut j2, mut k): (usize, usize, usize) = (i, i, 0);

            h1 = h >> 1; // h1 = h / 2
            c = c * FRAC_1_SQRT_2;
            let mut t: Vec<f32> = vec![0.0; h1];
            while k < h1 {
                let j21: usize = j2 + NUM_PIXELS;
                t[k] = (a[j2] - a[j21]) * c;
                a[j1] = a[j2] + a[j21];

                k += 1;
                j1 += NUM_PIXELS;
                j2 += NUM_PIXELS * 2;
            }

            // Write back subtraction results:
            let mut j1 = i + (h1 * NUM_PIXELS);
            for k in 0..h1 {
                a[j1] = t[k];
                j1 += NUM_PIXELS;
            }
            h = h1;
        }
        // Fix first element of each row:
        a[i] *= c;
    }
}

fn haar_2d(a: &mut Vec<f32>) {
    haar_rows(a);
    haar_columns(a);
}

fn rgb_to_yiq_conversion(img: DynamicImage) -> (Vec<f32>, Vec<f32>, Vec<f32>) {
    let mut cdata1: Vec<f32> = Vec::with_capacity(128 * 128);
    let mut cdata2: Vec<f32> = Vec::with_capacity(128 * 128);
    let mut cdata3: Vec<f32> = Vec::with_capacity(128 * 128);

    for p in img.pixels() {
        // The iteration order is x = 0 to width then y = 0 to height
        // RGB -> YIQ colorspace conversion; Y luminance, I,Q chrominance.
        // If RGB in [0..255] then Y in [0..255] and I,Q in [-127..127].
        let r: f32 = p.2[0] as f32;
        let g: f32 = p.2[1] as f32;
        let b: f32 = p.2[2] as f32;

        cdata1.push(0.299 * r + 0.587 * g + 0.114 * b);
        cdata2.push(0.596 * r - 0.275 * g - 0.321 * b);
        cdata3.push(0.212 * r - 0.523 * g + 0.311 * b);
    }

    (cdata1, cdata2, cdata3)
}

// Find the NUM_COEFS largest numbers in cdata[] (in magnitude that is)
// and store their indices in sig[].
fn get_m_largest(mut cdata: Vec<f32>) -> [i16; NUM_COEFS] {
    cdata.sort_by(|a, b| ((a.abs()).partial_cmp(&(b.abs())).unwrap()).reverse());

    let mut sig: [i16; NUM_COEFS] = [0; NUM_COEFS];
    for (val, s) in izip!(cdata.iter(), sig.iter_mut()) {
        *s = val.floor() as i16;
    }
    sig.sort();
    println!("sig: {:?}", sig);

    sig
}

// Determines a total of NUM_COEFS positions in the image that have the
// largest magnitude (absolute value) in color value. Returns linearized
// coordinates in sig1, sig2, and sig3. avgl are the [0,0] values.
// The order of occurrence of the coordinates in sig doesn't matter.
// Complexity is 3 x NUM_PIXELS^2 x 2log(NUM_COEFS).
pub fn calc_haar(cdata1: Vec<f32>, cdata2: Vec<f32>, cdata3: Vec<f32>) -> (LuminT, SignatureT) {
    let avglf: [f32; NUM_CHANNELS] = [cdata1[0], cdata2[0], cdata3[0]];

    // Color channel 1
    // Skip i=0, since it goes into avglf
    let sig1: [i16; NUM_COEFS] = get_m_largest(cdata1[1..].to_vec());

    // Color channel 2
    let sig2: [i16; NUM_COEFS] = get_m_largest(cdata2[1..].to_vec());

    // Color channel 3
    let sig3: [i16; NUM_COEFS] = get_m_largest(cdata2[1..].to_vec());

    (
        avglf,
        SignatureT {
            sig: [sig1, sig2, sig3],
        },
    )
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs::File;
    use std::io::{self, BufRead};
    use std::path::Path;

    const PATH: &str = "reference/";
    const RESIZE: [&str; 3] = ["r_resize.txt", "g_resize.txt", "b_resize.txt"];
    const ORIGINAL: [&str; 3] = ["r_buf.txt", "g_buf.txt", "b_buf.txt"];

    #[test]
    fn testreference() {
        let img = read_i32_vector_file("reference/b_original.txt".to_string());
        println!("test {:?}", img);
    }

    fn read_i32_vector_file(filename: String) -> Vec<u8> {
        let mut ret: Vec<u8> = vec![];
        if let Ok(lines) = read_lines(filename) {
            // Consumes the iterator, returns an (Optional) String
            // For these I'm using only 1 line
            for line in lines.flatten() {
                let iter = line.split(",");

                for value in iter {
                    match value.parse::<u8>() {
                        Ok(n) => ret.push(n),
                        Err(e) => println!("Parsing error for : {}, with error {}", value, e),
                    }
                }
            }
        }
        ret
    }

    fn compare_vals(v1: Vec<i32>, v2: Vec<f32>) {
        for (reference, val) in izip!(v1, v2) {
            assert_eq!(reference, val.trunc() as i32);
        }
    }

    fn compare_vals_ints(v1: Vec<i32>, v2: Vec<i32>) {
        for (reference, val) in izip!(v1, v2) {
            assert_eq!(reference, val);
        }
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
