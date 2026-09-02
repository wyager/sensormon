//! Unit newtypes. Mixing Hz with samples or dB with linear power is a
//! compile error, not a 3 a.m. debugging session.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::{Add, Div, Mul, Sub};

/// A frequency in hertz (absolute RF, or an offset from a center frequency).
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Hertz(pub f64);

impl Hertz {
    pub const fn khz(k: f64) -> Self {
        Hertz(k * 1e3)
    }
    pub const fn mhz(m: f64) -> Self {
        Hertz(m * 1e6)
    }
    pub fn abs(self) -> Self {
        Hertz(self.0.abs())
    }
}
impl Add for Hertz {
    type Output = Hertz;
    fn add(self, o: Hertz) -> Hertz {
        Hertz(self.0 + o.0)
    }
}
impl Sub for Hertz {
    type Output = Hertz;
    fn sub(self, o: Hertz) -> Hertz {
        Hertz(self.0 - o.0)
    }
}
impl Mul<f64> for Hertz {
    type Output = Hertz;
    fn mul(self, k: f64) -> Hertz {
        Hertz(self.0 * k)
    }
}
impl Div<f64> for Hertz {
    type Output = Hertz;
    fn div(self, k: f64) -> Hertz {
        Hertz(self.0 / k)
    }
}
impl fmt::Display for Hertz {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let v = self.0;
        if v.abs() >= 1e6 {
            write!(f, "{:.4} MHz", v / 1e6)
        } else if v.abs() >= 1e3 {
            write!(f, "{:.1} kHz", v / 1e3)
        } else {
            write!(f, "{:.0} Hz", v)
        }
    }
}

/// Samples per second of a stream.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SampleRate(pub u32);

impl SampleRate {
    pub fn hz(self) -> f64 {
        self.0 as f64
    }
    /// Number of samples spanning `seconds`.
    pub fn samples_in(self, seconds: f64) -> usize {
        (self.hz() * seconds).round() as usize
    }
    /// Seconds spanned by `n` samples.
    pub fn seconds_of(self, n: u64) -> f64 {
        n as f64 / self.hz()
    }
}
impl fmt::Display for SampleRate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.3} MS/s", self.hz() / 1e6)
    }
}

/// Absolute position in a receiver's sample stream, counted from stream start.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SampleIndex(pub u64);

impl SampleIndex {
    pub fn offset(self, n: i64) -> SampleIndex {
        SampleIndex((self.0 as i64 + n).max(0) as u64)
    }
    pub fn distance_to(self, later: SampleIndex) -> u64 {
        later.0.saturating_sub(self.0)
    }
}

/// A level or ratio in decibels. Power levels are relative to full scale (dBFS).
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Db(pub f32);

impl Db {
    pub fn from_power_ratio(p: f32) -> Db {
        Db(10.0 * p.max(1e-30).log10())
    }
    pub fn to_power_ratio(self) -> f32 {
        10f32.powf(self.0 / 10.0)
    }
}
impl Add for Db {
    type Output = Db;
    fn add(self, o: Db) -> Db {
        Db(self.0 + o.0)
    }
}
impl Sub for Db {
    type Output = Db;
    fn sub(self, o: Db) -> Db {
        Db(self.0 - o.0)
    }
}
impl fmt::Display for Db {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.1} dB", self.0)
    }
}
