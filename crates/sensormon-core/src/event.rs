//! Typed events. Control plane (who heard it, when, how well) and data plane
//! (what the sensor said) are separate structs; a decoded reading never
//! carries RSSI fields next to temperature fields.

use crate::units::{Db, Hertz};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;

/// Name of a receiver pipeline, e.g. "tower-airspy". Configured by the operator.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ReceiverId(pub String);

impl fmt::Display for ReceiverId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// How one receiver saw one transmission.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Signal {
    /// Absolute RF center of the burst as detected (mean of the two FSK tones).
    pub center: Hertz,
    /// Absolute RF frequency of the "1"/mark tone and "0"/space tone.
    pub f_mark: Hertz,
    pub f_space: Hertz,
    /// Measured symbol rate after timing recovery.
    pub symbol_rate: Hertz,
    /// Burst power relative to full scale.
    pub rssi: Db,
    /// Burst power over the noise floor in the burst's own bandwidth.
    pub snr: Db,
    /// Noise floor (dBFS per bin) at the burst's frequency at the time.
    pub noise: Db,
}

/// Control plane: when a transmission was received and by whom.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Reception {
    /// Wall-clock time of the earliest receiver's copy.
    pub time: DateTime<Utc>,
    /// Every receiver that decoded this exact transmission, with its own signal stats.
    pub heard_by: BTreeMap<ReceiverId, Signal>,
}

/// A decoded transmission from one receiver, before cross-receiver merging.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReceiverEvent {
    pub receiver: ReceiverId,
    pub time: DateTime<Utc>,
    pub signal: Signal,
    pub sensor: Payload,
    /// Frame bytes after the sync word, integrity bytes included. Identical
    /// bytes from two receivers within a short window are the same transmission.
    pub raw: Vec<u8>,
}

/// A transmission as reported to consumers: one sensor reading, heard by one
/// or more receivers.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Event {
    pub reception: Reception,
    pub sensor: Payload,
    pub raw: Vec<u8>,
}

/// Data plane: what a sensor reported. Fields that the sensor can flag as
/// "not available" are `Option`s. Units are in the field names.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "model")]
pub enum Payload {
    #[serde(rename = "Fineoffset-WS90")]
    Ws90(Ws90),
    #[serde(rename = "Fineoffset-WH51")]
    Wh51(Wh51),
    #[serde(rename = "Fineoffset-WH55")]
    Wh55(Wh55),
    #[serde(rename = "FineOffset-WH31L")]
    Wh31l(Wh31l),
    #[serde(rename = "Fineoffset-WH25")]
    Wh25(Wh25),
    #[serde(rename = "Toyota-TPMS")]
    ToyotaTpms(ToyotaTpms),
}

impl Payload {
    /// rtl_433-compatible model name.
    pub fn model(&self) -> &'static str {
        match self {
            Payload::Ws90(_) => "Fineoffset-WS90",
            Payload::Wh51(_) => "Fineoffset-WH51",
            Payload::Wh55(_) => "Fineoffset-WH55",
            Payload::Wh31l(_) => "FineOffset-WH31L",
            Payload::Wh25(_) => "Fineoffset-WH25",
            Payload::ToyotaTpms(_) => "Toyota-TPMS",
        }
    }
    /// The sensor's own identifier (width depends on the model).
    pub fn sensor_id(&self) -> u32 {
        match self {
            Payload::Ws90(p) => p.id,
            Payload::Wh51(p) => p.id,
            Payload::Wh55(p) => p.id as u32,
            Payload::Wh31l(p) => p.id,
            Payload::Wh25(p) => p.id as u32,
            Payload::ToyotaTpms(p) => p.id,
        }
    }
}

/// Fineoffset WS90 7-in-1 weather station.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Ws90 {
    pub id: u32,
    pub battery_mv: u16,
    /// 0.0..=1.0, mapped from 1.4 V..3.0 V.
    pub battery_level: f32,
    pub temperature_c: Option<f32>,
    pub humidity_pct: Option<u8>,
    pub pressure_hpa: Option<f32>,
    pub wind_dir_deg: Option<u16>,
    pub wind_avg_m_s: Option<f32>,
    pub wind_max_m_s: Option<f32>,
    pub uv_index: Option<f32>,
    pub light_lux: Option<f32>,
    /// Lifetime rain counter.
    pub rain_total_mm: f32,
    pub rain_start: bool,
    pub supercap_v: Option<f32>,
    pub firmware: u8,
    pub flags: u8,
}

/// Fineoffset WH51 soil moisture probe.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Wh51 {
    /// 24-bit id; rtl_433 prints it as 6 hex digits.
    pub id: u32,
    pub battery_mv: u16,
    /// 0.0..=1.0 per rtl_433's alkaline-cell mapping.
    pub battery_level: f32,
    pub moisture_pct: u8,
    pub boost: u8,
    pub ad_raw: u16,
}

/// Fineoffset WH55 water leak sensor.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Wh55 {
    pub id: u16,
    pub channel: u8,
    /// 0.2 steps, 0.2..=1.0.
    pub battery_level: f32,
    pub raw_value: u16,
    pub high_sensitivity: bool,
    pub alarm: bool,
}

/// Fineoffset WH57 lightning sensor (rtl_433 calls it FineOffset-WH31L).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Wh31l {
    pub id: u32,
    /// 0.0, 0.5 or 1.0.
    pub battery_level: f32,
    pub state: LightningState,
    pub storm_dist_km: Option<u8>,
    pub strike_count: u8,
    pub flags: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LightningState {
    Reset,
    Interference,
    Noise,
    Strike,
    Unknown(u8),
}

/// Fineoffset WH25 / WH32 / WH32B indoor temperature, humidity, pressure.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Wh25 {
    pub id: u8,
    pub variant: Wh25Variant,
    pub battery_ok: bool,
    pub temperature_c: Option<f32>,
    pub humidity_pct: u8,
    pub pressure_hpa: Option<f32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Wh25Variant {
    Wh25,
    Wh32,
    Wh32b,
}

/// Toyota / Pacific Industries PMV-C210 tire pressure sensor (315 MHz in the US).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ToyotaTpms {
    /// 32-bit sensor id (rtl_433 prints 8 hex digits).
    pub id: u32,
    pub pressure_psi: f32,
    pub temperature_c: f32,
    /// Raw status byte (bit 7 = state flag, low 7 bits as transmitted).
    pub status: u8,
}
