//! Signal processing. All functions are pure over slices; the only stateful
//! items are explicit `*State`/detector structs owned by the caller.

pub mod burst;
pub mod extract;
pub mod fir;
pub mod fsk;
pub mod ring;

use num_complex::Complex;

/// One complex baseband sample, full scale = magnitude 1.0.
pub type Iq = Complex<f32>;

/// Mean power of a block, in dB relative to full scale.
pub fn power_dbfs(x: &[Iq]) -> crate::units::Db {
    if x.is_empty() {
        return crate::units::Db(-200.0);
    }
    let p = x.iter().map(|s| s.norm_sqr()).sum::<f32>() / x.len() as f32;
    crate::units::Db::from_power_ratio(p)
}
