use rand_chacha::ChaChaRng;
use substrate_fixed::types::I32F32;
use substrate_fixed::transcendental::ln;
use crate::random_unit_interval;
  
pub fn laplace(
    rng: &mut ChaChaRng,
    scale: I32F32,
) -> I32F32 {
    let e1: I32F32 = (-scale) * ln::<I32F32, I32F32>(random_unit_interval(rng)).unwrap();
    let e2: I32F32 = (-scale) * ln::<I32F32, I32F32>(random_unit_interval(rng)).unwrap();
    e1 - e2
}
