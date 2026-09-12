//! `sensormon.toml`.

use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(rename = "receiver")]
    pub receivers: Vec<ReceiverConfig>,
    #[serde(default)]
    pub merge: MergeConfig,
    #[serde(default)]
    pub http: HttpConfig,
    /// Rolling store of undecoded bursts (see chirps.rs); absent = off.
    pub chirps: Option<crate::chirps::ChirpStoreConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ReceiverConfig {
    pub name: String,
    #[serde(flatten)]
    pub kind: ReceiverKind,
    /// Center frequency; with `hop_hz` this is ignored in favour of the list.
    #[serde(default)]
    pub center_hz: Option<f64>,
    /// Frequencies to cycle through (time-multiplexed), each for `dwell_s`.
    #[serde(default)]
    pub hop_hz: Vec<f64>,
    #[serde(default = "default_dwell")]
    pub dwell_s: f64,
    /// Per-band pipelines kept alive (LRU); a band evicted and revisited
    /// re-learns its noise floor in a few ms. Default 8.
    #[serde(default = "default_max_bands")]
    pub max_band_pipelines: usize,
    pub sample_rate: u32,
}

fn default_max_bands() -> usize {
    8
}

fn default_dwell() -> f64 {
    15.0
}

impl ReceiverConfig {
    /// The centers this receiver visits, in order.
    pub fn centers(&self) -> Vec<f64> {
        if !self.hop_hz.is_empty() {
            self.hop_hz.clone()
        } else {
            self.center_hz.into_iter().collect()
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum ReceiverKind {
    Airspy {
        /// Serial number (hex, as printed by airspy_info); first device if absent.
        serial: Option<String>,
        gain: AirspyGain,
        #[serde(default)]
        bias_tee: bool,
    },
    Rtlsdr {
        /// Device index (default 0).
        #[serde(default)]
        index: u32,
        gain: RtlsdrGain,
        #[serde(default)]
        bias_tee: bool,
    },
    /// Replay a recorded IQ file in real time (testing).
    File {
        path: String,
        format: FileFormat,
        #[serde(default = "default_true")]
        realtime: bool,
    },
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(untagged)]
pub enum AirspyGain {
    /// Individual stages, each 0..=15.
    Stages { lna: u8, mix: u8, vga: u8 },
    /// Airspy's combined presets, 0..=21.
    Preset { linearity: Option<u8>, sensitivity: Option<u8> },
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(untagged)]
pub enum RtlsdrGain {
    Auto(AutoWord),
    Db(f32),
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AutoWord {
    Auto,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileFormat {
    Cu8,
    Cs16,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MergeConfig {
    pub window_ms: u64,
}

impl Default for MergeConfig {
    fn default() -> Self {
        MergeConfig { window_ms: 2000 }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HttpConfig {
    pub bind: String,
}

impl Default for HttpConfig {
    fn default() -> Self {
        HttpConfig { bind: "0.0.0.0:8433".into() }
    }
}

pub fn load(path: &Path) -> Result<Config> {
    let text = std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let cfg: Config = toml::from_str(&text).with_context(|| format!("parse {}", path.display()))?;
    if cfg.receivers.is_empty() {
        anyhow::bail!("no [[receiver]] entries");
    }
    for r in &cfg.receivers {
        if r.centers().is_empty() {
            anyhow::bail!("receiver {}: give center_hz or hop_hz", r.name);
        }
        if r.hop_hz.len() > 1 && !matches!(r.kind, ReceiverKind::Rtlsdr { .. }) {
            anyhow::bail!("receiver {}: hopping is only implemented for rtlsdr", r.name);
        }
    }
    Ok(cfg)
}
