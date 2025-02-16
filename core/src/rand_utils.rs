use std::ops::Range;

use shared_deps::bevy_turborand::DelegatedRng;

pub fn gen_f32_range<T: DelegatedRng>(rng: &mut T, range: &Range<f32>) -> f32 {
    const MULTIPLY_FACTOR: f32 = 1000.0;
    rng.i32((range.start * MULTIPLY_FACTOR) as i32..(range.end * MULTIPLY_FACTOR) as i32) as f32
        / MULTIPLY_FACTOR
}
