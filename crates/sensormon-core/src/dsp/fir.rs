//! Windowed-sinc low-pass FIR design and filter-and-decimate.

use super::Iq;
use std::f32::consts::PI;

/// Low-pass taps with cutoff as a fraction of the sample rate (0 < cutoff < 0.5),
/// Kaiser-windowed, unity DC gain.
pub fn lowpass_taps(cutoff_frac: f32, ntaps: usize, kaiser_beta: f32) -> Vec<f32> {
    assert!(ntaps >= 3 && cutoff_frac > 0.0 && cutoff_frac < 0.5);
    let m = (ntaps - 1) as f32;
    let mut taps: Vec<f32> = (0..ntaps)
        .map(|n| {
            let x = n as f32 - m / 2.0;
            let sinc = if x == 0.0 { 2.0 * cutoff_frac } else { (2.0 * PI * cutoff_frac * x).sin() / (PI * x) };
            let r = 2.0 * n as f32 / m - 1.0;
            sinc * bessel_i0(kaiser_beta * (1.0 - r * r).max(0.0).sqrt()) / bessel_i0(kaiser_beta)
        })
        .collect();
    let sum: f32 = taps.iter().sum();
    taps.iter_mut().for_each(|t| *t /= sum);
    taps
}

fn bessel_i0(x: f32) -> f32 {
    let mut sum = 1.0f32;
    let mut term = 1.0f32;
    let q = x * x / 4.0;
    for k in 1..30 {
        term *= q / (k * k) as f32;
        sum += term;
        if term < 1e-9 * sum {
            break;
        }
    }
    sum
}

/// Apply `taps` and keep every `decim`-th output. Output sample `k` is the
/// filter centered on input `k * decim + ntaps/2` (group delay compensated
/// by starting at the first fully-supported position).
pub fn filter_decimate(x: &[Iq], taps: &[f32], decim: usize) -> Vec<Iq> {
    let n = taps.len();
    if x.len() < n {
        return Vec::new();
    }
    (0..=(x.len() - n) / decim)
        .map(|k| {
            let base = k * decim;
            let mut acc = Iq::new(0.0, 0.0);
            for (t, &h) in taps.iter().enumerate() {
                acc += x[base + t] * h;
            }
            acc
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn passes_dc_rejects_high_tone() {
        let taps = lowpass_taps(0.05, 63, 8.0);
        assert!((taps.iter().sum::<f32>() - 1.0).abs() < 1e-5);
        let dc: Vec<Iq> = (0..400).map(|_| Iq::new(1.0, 0.0)).collect();
        let y = filter_decimate(&dc, &taps, 4);
        assert!(y.iter().all(|s| (s.re - 1.0).abs() < 1e-4));
        let tone: Vec<Iq> = (0..400).map(|n| Iq::from_polar(1.0, 2.0 * PI * 0.3 * n as f32)).collect();
        let y = filter_decimate(&tone, &taps, 4);
        assert!(y.iter().all(|s| s.norm() < 1e-2));
    }
}
