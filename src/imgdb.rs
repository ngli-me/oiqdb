use image::DynamicImage;
use crate::signature::haar::{Idx, NUM_COEFS, NUM_PIXELS, NUM_PIXELS_SQUARED};
use crate::signature::{haar, HaarSignature};
use num_traits::abs;
use std::borrow::Borrow;
use std::cmp::{max, min};
use std::collections::BinaryHeap;

type ImageId = i32;
type IqdbId = usize; // An internal IQDB image ID.
type PostId = i32; // An external (booru) post ID.
type Score = f32;
type SimVector = Vec<SimValue>;
type Sig = [Idx; NUM_COEFS];
type Bucket = Vec<u32>;

struct ImageInfo {
    id: ImageId,
    avgl: LuminNative,
}

struct LuminNative {
    pub v: [Score; 3],
}

struct SimValue {
    pub id: ImageId,
    pub score: Score,
}

const N_SIGNS: usize = 2; // 2 haar coefficient signs (positive and negative)
const N_INDEXES: usize = NUM_PIXELS_SQUARED; // 16384 haar matrix indices

// Weights for the Haar coefficients, straight from the referenced paper:
pub const WEIGHTS: [&[f32; 3]; 6] = [
    // For scanned picture (sketch=0):
    //    Y      I      Q       idx total occurs
    &[5.00, 19.21, 34.37], // 0   58.58      1 (`DC' component)
    &[0.83, 01.26, 00.36], // 1    2.45      3
    &[1.01, 00.44, 00.45], // 2    1.90      5
    &[0.52, 00.53, 00.14], // 3    1.19      7
    &[0.47, 00.28, 00.18], // 4    0.93      9
    &[0.30, 00.14, 00.27], // 5    0.71      16384-25=16359
];

pub struct ImgBin {
    // A 128x128 weight mask matrix, where M[x][y] = min(max(x, y), 5). Used in score calculation.
    // 0 1 2 3 4 5 5 ...
    // 1 1 2 3 4 5 5 ...
    // 2 2 2 3 4 5 5 ...
    // 3 3 3 3 4 5 5 ...
    // 4 4 4 4 4 5 5 ...
    // 5 5 5 5 5 5 5 ...
    // 5 5 5 5 5 5 5 ...
    // . . . . . . .
    // . . . . . . .
    // . . . . . . .
    bin: [usize; NUM_PIXELS * NUM_PIXELS],
    buckets: Vec<Vec<Vec<Bucket>>>,
    info: Vec<ImageInfo>,
}

impl ImgBin {
    pub fn new() -> Self {
        let mut bin: [usize; NUM_PIXELS * NUM_PIXELS] = [0; NUM_PIXELS * NUM_PIXELS];
        let mut i = 0;
        let mut j: usize;
        while i < NUM_PIXELS {
            j = 0;
            while j < NUM_PIXELS {
                bin[(i * NUM_PIXELS) + j] = min(max(i, j), 5);
                j += 1;
            }
            i += 1;
        }
        Self {
            bin: bin,
            buckets: vec![vec![vec![Vec::new(); N_INDEXES]; N_SIGNS]; haar::N_COLORS], // 3 * 2 * 16384 = 98304 total buckets
            info: Vec::new(),
        }
    }

    fn at(&mut self, color: usize, coef: i16) -> &mut Bucket {
        let sign: bool = coef < 0;
        &mut self.buckets[color][sign as usize][abs(coef) as usize]
    }

    fn each_bucket<F>(&mut self, sig: &HaarSignature, func: F) where
        F: Fn(&mut Bucket) {
        for c in 0..sig.num_colors() {
            for i in 0..NUM_COEFS {
                let coef: i16 = sig[c][i];
                let bucket: &mut Bucket = self.at(c, coef);
                func(bucket)
            }
        }
    }

    // addImage
    // addImageInMemory
    // loadDatabase

    fn is_deleted(&self, iqdb_id: IqdbId) -> bool {
        return self.info[iqdb_id].avgl.v[0] == 0.0;
    }

    pub fn add(&mut self, sig: &HaarSignature, iqdb_id: u32) {
        self.each_bucket(sig, |bucket: &mut Bucket| { bucket.push(iqdb_id) });
    }

    pub fn remove(&mut self, sig: &HaarSignature, iqdb_id: u32) {
        self.each_bucket(sig, |bucket: &mut Bucket| { bucket.retain(|&x: &u32| x != iqdb_id) })
    }

    fn query_from_blob(&self, image: DynamicImage, limit: i32) {
        let signature: HaarSignature = HaarSignature::from(image);
    }

    /*fn query_from_signature(&mut self, signature: &HaarSignature, num_res: usize) {
        // results priority queue; largest at top
        let v: SimVector = Vec::new();

        let scores: Vec<Score> = vec![0.0; self.info.len()];
        // Luminance score (DC coefficient)
        for i in 0..scores.len() {
            let image_info: &ImageInfo = &self.info[i];
            let mut s: Score = 0.0;

            for c in 0..signature.num_colors() {
                s += WEIGHTS[0][c] * abs(image_info.avgl.v[c] - signature.avglf[c]);
            }

            scores[i] = s;
        }

        let mut scale: Score = 0.0;
        for c in 0..signature.num_colors() {
            for b in 0..NUM_COEFS { // for every coef on a sig
                let coef: i16 = signature[c][b];
                let bucket: &mut Bucket = self.at(c, coef);
                if bucket.is_empty() {
                    continue;
                }
                let w: usize = self.bin[abs(coef)];
                let weight: Score = WEIGHTS[w][c];
                scale -= weight;
                for index in bucket.iter() {
                    scores[index] -= weight;
                }
            }
        }

        // Fill up the numres-bounded priority queue (largest at top):
        let mut i: IqdbId = 0;
        let pq_results: BinaryHeap<SimValue> = BinaryHeap::new();
        while (pq_results.len() < num_res) && (i < scores.len()) {
            if !self.is_deleted(i) {}
            i += 1;
        }
    }*/
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn max_paths() {
        let img_bin: ImgBin = ImgBin::new();
    }
}
