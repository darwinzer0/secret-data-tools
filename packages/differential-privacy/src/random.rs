use rand::RngCore;
use rand_chacha::ChaChaRng;
use substrate_fixed::types::{I32F32,I64F64};

// returns a random fixed point number between 0..1 as I32F32
pub fn random_unit_interval(rng: &mut ChaChaRng) -> I32F32 {
    let numerator = rng.next_u32();
    let ratio = I64F64::from_num(numerator) / I64F64::from_num(u32::MAX);
    I32F32::from_num(ratio)
}