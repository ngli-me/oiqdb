use image::{DynamicImage, GenericImageView};
use std::cmp::Ordering;
//use itertools::izip;

pub const NUM_PIXELS: usize = 128;
pub const NUM_PIXELS_SQUARED: usize = NUM_PIXELS.pow(2);
pub const NUM_COEFS: usize = 40;

struct ValStruct {
    v: f32,
}

impl PartialOrd for ValStruct {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let v1 = f32::abs(self.v);
        let v2 = f32::abs(other.v);
        v1.partial_cmp(&v2)
    }
}

impl PartialEq for ValStruct {
    fn eq(&self, other: &Self) -> bool {
        let v1 = f32::abs(self.v);
        let v2 = f32::abs(other.v);
        v1 == v2
    }
}

pub type LuminT = [f32; 3];
pub struct SignatureT ([i16; 3 * NUM_COEFS]);

impl Default for SignatureT {
    fn default() -> Self {
        SignatureT([0; 3 * NUM_COEFS])
    }
}

pub async fn transform_char(img: DynamicImage) ->
    (Vec<f32>, Vec<f32>, Vec<f32>){

    let (mut a, mut b, mut c) = rgb_to_yiq_conversion(img).await;
    haar_2d(&mut a).await;
    haar_2d(&mut b).await;
    haar_2d(&mut c).await;

    // Reintroduce the skipped scaling factors
    a[0] /= 256.0 * 128.0;
    b[0] /= 256.0 * 128.0;
    c[0] /= 256.0 * 128.0;

    (a, b, c)
}

async fn haar_2d(a: &mut Vec<f32>) {
    // Rows
    for i in (0..NUM_PIXELS_SQUARED).step_by(NUM_PIXELS) {
        let (mut h, mut h1): (usize, usize);
        let mut c: f32 = 1.0;
        h = NUM_PIXELS;
        while h > 1 {
            let (mut j1, mut j2, mut k): (usize, usize, usize) = (i, i, 0);

            h1 = h >> 1; // h1 = h / 2
            c = c * 0.7071; // 1/sqrt(2)
            let mut t: Vec<f32> = vec![0.0; h1];
            while k < h1 {
                let j21: usize = j2 + 1;
                t[k]  = (a[j2] - a[j21]) * c;
                a[j1] = a[j2] + a[j21];

                k  += 1;
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

    // Columns
    for i in 0..NUM_PIXELS {
        let (mut h, mut h1): (usize, usize);
        let mut c: f32 = 1.0;
        h = NUM_PIXELS;
        while h > 1 {
            let (mut j1, mut j2, mut k): (usize, usize, usize) = (i, i, 0);

            h1 = h >> 1; // h1 = h / 2
            c = c * 0.7071; // 1/sqrt(2)
            let mut t: Vec<f32> = vec![0.0; h1];
            while k < h1 {
                let j21: usize = j2 + NUM_PIXELS;
                t[k]  = (a[j2] - a[j21]) * c;
                a[j1] = a[j2] + a[j21];

                k  += 1;
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

async fn rgb_to_yiq_conversion(img: DynamicImage) ->
    (Vec<f32>, Vec<f32>, Vec<f32>) {

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
async fn get_m_largest(mut cdata: Vec<f32>) -> Vec<i16> {
    let mut cnt: usize = 0;
    let i: i16 = 0;
    let mut vq: Vec<ValStruct> = Vec::new();

    // Skip i = 0, since it goes into avglf
    for val in cdata.iter_mut().skip(1) {
        vq.push(ValStruct { v: *val });
    }

    vq.sort_by(|a, b| a.partial_cmp(b).unwrap());

    // Set sorted size to NUM_COEFS, discard others
    let vq = &vq[..NUM_COEFS];

    // The index is positive if the coefficient was positive; the index is
    // negative if the coefficient was negative.
    for val in vq {

    }

    vec![0; NUM_PIXELS]
}

// Determines a total of NUM_COEFS positions in the image that have the
// largest magnitude (absolute value) in color value. Returns linearized
// coordinates in sig1, sig2, and sig3. avgl are the [0,0] values.
// The order of occurrence of the coordinates in sig doesn't matter.
// Complexity is 3 x NUM_PIXELS^2 x 2log(NUM_COEFS).
pub async fn calc_haar(cdata1: Vec<f32>, cdata2: Vec<f32>, cdata3: Vec<f32>) {//->
    //(LuminT, SignatureT) {
    let avglf: Vec<f32> = vec![cdata1[0], cdata2[0], cdata3[0]];

    // Color channel 1
    let mut c: Vec<i16>  = get_m_largest(cdata1).await;

    // Color channel 2
    //c.append(get_m_largest(cdata2));

    // Color channel 3
    //c.append(get_m_largest(cdata2));

    //(avglf, SignatureT{[i16; 128 * 3]})
}
