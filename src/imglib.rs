use num_traits::abs;
use crate::signature::haar::{NUM_COEFS, NUM_PIXELS, NUM_PIXELS_SQUARED};
use crate::signature::HaarSignature;

type BucketT = Vec<i32>;
type Score = f32;

const N_COLORS: usize = 3;            // 3 color channels (YIQ)
const N_SIGNS: usize = 2;             // 2 haar coefficient signs (positive and negative)
const N_INDEXES: usize = NUM_PIXELS_SQUARED; // 16384 haar matrix indices

// Weights for the Haar coefficients.
// Straight from the referenced paper:
pub const WEIGHTS: [&[f32; 3]; 6] = [
    // For scanned picture (sketch=0):
    //    Y      I      Q       idx total occurs
    &[5.00, 19.21, 34.37],   // 0   58.58      1 (`DC' component)
    &[0.83, 01.26, 00.36],   // 1    2.45      3
    &[1.01, 00.44, 00.45],   // 2    1.90      5
    &[0.52, 00.53, 00.14],   // 3    1.19      7
    &[0.47, 00.28, 00.18],   // 4    0.93      9
    &[0.30, 00.14, 00.27]];  // 5    0.71      16384-25=16359

static BUCKETS: Vec<Vec<Vec<BucketT>>> = vec![vec![vec![Vec::new(); N_INDEXES]; N_SIGNS]; N_COLORS]; // 3 * 2 * 16384 = 98304 total buckets

// A 128x128 weight mask matrix, where M[x][y] = min(max(x, y), 5). Used in
// score calculation.
//
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
static IMG_BIN: [usize; NUM_PIXELS * NUM_PIXELS] = initialize_imgbin();

/*fn add(sig: &HaarSignature, iqdb_id: u32) {
    each_bucket(sig);
}*/

fn erase() {}

fn at<'a>(color: i32, coef: i16) -> &'a BucketT {
    const SIGN: bool = coef < 0;
    BUCKETS[color as u32][SIGN][abs(coef)]
}

fn each_bucket(sig: &HaarSignature, func: fn(bucket_t: &BucketT)) {
    for c in 0..sig.num_colors() {
        for i in 0..NUM_COEFS {
            let coef: i16 = sig.sig[c][i];
            let bucket: &BucketT = at(c, coef);
            func(bucket)
        }
    }
}

const fn initialize_imgbin() -> [usize; NUM_PIXELS * NUM_PIXELS] {
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
    bin
}

// Min/Max don't have const variants, so we have to make our own
const fn max(v1: usize, v2: usize) -> usize {
    if v1 >= v2 {
        v1
    } else {
        v2
    }
}

const fn min(v1: usize, v2: usize) -> usize {
    if v1 <= v2 {
        v1
    } else {
        v2
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn max_paths() {
        let x: usize = 1;
        let y: usize = 2;
        assert_eq!(y, max(x, y));
        assert_eq!(y, max(y, x));
        assert_eq!(y, max(y, y));
    }

    #[test]
    fn min_paths() {
        let x: usize = 1;
        let y: usize = 2;
        assert_eq!(x, min(x, y));
        assert_eq!(x, min(y, x));
        assert_eq!(y, min(y, y));
    }
}
