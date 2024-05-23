use crate::signature::haar::{Idx, NUM_COEFS};

type ImageId = i32;
type IqdbId = i32; // An internal IQDB image ID.
type PostId = i32; // An external (booru) post ID.
type Score = f32;
type SimVector = Vec<SimValue>;
type SigT = [Idx; NUM_COEFS];

struct ImageInfo {
    id: ImageId,
    avgl: LuminNative,
}

struct LuminNative {
    pub v: [Score; 3],
}

struct SimValue {
    pub id: ImageId,
    pub avg1: LuminNative,
}

static mut M_INFO: Vec<ImageInfo> = Vec::new();

fn query_from_signature() {}

#[cfg(test)]
mod tests {}
