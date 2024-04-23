use crate::signature::haar::{Idx, NUM_COEFS, NUM_PIXELS};

type Score = f32;
type ImageId = i32;
// An external (Danbooru) post ID.
type PostId = i32;
// An internal IQDB image ID.
type IqdbId = i32;

struct LuminNative {
    pub v: [Score; 3],
}

struct SimValue {
    pub id: ImageId,
    pub avg1: LuminNative,
}

type SimVector = Vec<SimValue>;
type SigT = [Idx; NUM_COEFS];

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
    if v1 > v2 {
        v1
    } else {
        v2
    }
}

const fn min(v1: usize, v2: usize) -> usize {
    if v1 < v2 {
        v1
    } else {
        v2
    }
}

fn query_from_signature() {}