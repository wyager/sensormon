//! Rolling store of "chirps": bursts that decoded to nothing. Two tables:
//! `groups` aggregates recurring emitters (receiver + frequency + bandwidth +
//! duration class) with counts and the demodulator's best guess, and
//! `chirps` keeps a bounded number of example IQ captures per group under a
//! global byte cap (oldest evicted first). The point is to be able to come
//! back later and identify everything that talks on the unlicensed bands.

use anyhow::{Context, Result};
use rusqlite::{params, Connection, OptionalExtension};
use sensormon_core::pipeline::Chirp;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChirpStoreConfig {
    pub path: String,
    /// Global cap on stored IQ bytes; oldest chirps are evicted past it.
    #[serde(default = "default_max_bytes")]
    pub max_bytes: i64,
    /// Newest examples kept per group.
    #[serde(default = "default_examples")]
    pub max_examples_per_group: i64,
    /// Only record these receivers (empty = all).
    #[serde(default)]
    pub receivers: Vec<String>,
    /// Newest examples kept per emitter family (`Chirp::family`: receiver,
    /// bandwidth/duration/tone-spacing/symbol-rate classes, any center), so a
    /// frequency hopper can't fill the store. Default 50.
    #[serde(default = "default_family_examples")]
    pub max_examples_per_family: i64,
    /// Receiver-side admission: at most one example per family per this many
    /// seconds reaches the store (group counts are unaffected). Default 10 s.
    #[serde(default = "default_family_interval")]
    pub min_family_interval_s: f64,
}

fn default_family_examples() -> i64 {
    50
}

fn default_family_interval() -> f64 {
    10.0
}
fn default_max_bytes() -> i64 {
    50_000_000
}
fn default_examples() -> i64 {
    6
}

/// Aggregate row for one recurring emitter class.
#[derive(Debug, Clone, Serialize)]
pub struct GroupRow {
    pub id: i64,
    pub receiver: String,
    /// Center rounded to 10 kHz.
    pub center_hz: i64,
    pub bandwidth_khz: i64,
    pub duration_class_ms: i64,
    pub count: i64,
    pub first_seen: f64,
    pub last_seen: f64,
    pub mean_snr_db: f64,
    pub fsk: bool,
    pub tone_spacing_hz: Option<f64>,
    pub symbol_rate: Option<f64>,
    pub last_bits: Option<String>,
    pub examples: i64,
    /// Median gap between sightings (seconds) over the last few — the "cadence".
    pub typical_period_s: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChirpMeta {
    pub id: i64,
    pub group_id: i64,
    pub receiver: String,
    pub time: f64,
    pub center_hz: f64,
    pub bandwidth_hz: f64,
    pub duration_ms: f64,
    pub peak_db: f64,
    pub noise_db: f64,
    pub snr_db: f64,
    pub f_mark_hz: Option<f64>,
    pub f_space_hz: Option<f64>,
    pub symbol_rate: Option<f64>,
    pub bits: Option<String>,
    pub sample_rate: i64,
    pub samples: i64,
    pub bytes: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct StoreStats {
    pub chirps: i64,
    pub groups: i64,
    pub bytes: i64,
    pub max_bytes: i64,
    pub oldest: Option<f64>,
    pub newest: Option<f64>,
}

pub struct ChirpStore {
    conn: Connection,
    cfg: ChirpStoreConfig,
    total_bytes: i64,
}

use sensormon_core::pipeline::duration_class_ms;

impl ChirpStore {
    pub fn open(cfg: ChirpStoreConfig) -> Result<Self> {
        if let Some(dir) = Path::new(&cfg.path).parent() {
            std::fs::create_dir_all(dir).ok();
        }
        let conn = Connection::open(&cfg.path).with_context(|| format!("open chirp db {}", cfg.path))?;
        conn.execute_batch(
            "PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;
             CREATE TABLE IF NOT EXISTS groups (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                receiver TEXT NOT NULL, center_hz INTEGER NOT NULL, bandwidth_khz INTEGER NOT NULL, duration_class_ms INTEGER NOT NULL,
                count INTEGER NOT NULL, first_seen REAL NOT NULL, last_seen REAL NOT NULL, sum_snr REAL NOT NULL,
                fsk INTEGER NOT NULL, tone_spacing_hz REAL, symbol_rate REAL, last_bits TEXT,
                recent_times TEXT NOT NULL DEFAULT '[]',
                UNIQUE(receiver, center_hz, bandwidth_khz, duration_class_ms));
             CREATE TABLE IF NOT EXISTS chirps (
                id INTEGER PRIMARY KEY AUTOINCREMENT, group_id INTEGER NOT NULL, receiver TEXT NOT NULL, time REAL NOT NULL,
                center_hz REAL NOT NULL, bandwidth_hz REAL NOT NULL, duration_ms REAL NOT NULL,
                peak_db REAL NOT NULL, noise_db REAL NOT NULL, snr_db REAL NOT NULL,
                f_mark_hz REAL, f_space_hz REAL, symbol_rate REAL, bits TEXT,
                sample_rate INTEGER NOT NULL, samples INTEGER NOT NULL, bytes INTEGER NOT NULL, iq BLOB NOT NULL);
             CREATE INDEX IF NOT EXISTS chirps_group ON chirps(group_id, id);
             CREATE INDEX IF NOT EXISTS chirps_time ON chirps(time);",
        )?;
        // family column (added 2026-09-05); older stores get it on open
        let has_family: bool = conn.prepare("PRAGMA table_info(chirps)")?.query_map([], |r| r.get::<_, String>(1))?.filter_map(|r| r.ok()).any(|c| c == "family");
        if !has_family {
            conn.execute_batch("ALTER TABLE chirps ADD COLUMN family TEXT NOT NULL DEFAULT ''")?;
        }
        conn.execute_batch("CREATE INDEX IF NOT EXISTS chirps_family ON chirps(family, id);")?;
        let total_bytes: i64 = conn.query_row("SELECT COALESCE(SUM(bytes),0) FROM chirps", [], |r| r.get(0))?;
        Ok(ChirpStore { conn, cfg, total_bytes })
    }

    pub fn insert(&mut self, c: &Chirp) -> Result<()> {
        let t = c.time.timestamp() as f64 + c.time.timestamp_subsec_micros() as f64 / 1e6;
        let center = c.burst.center().0;
        let bw = c.burst.bandwidth().0;
        let dur_ms = c.duration_ms();
        let family = c.family();
        let snr = (c.burst.peak.0 - c.burst.noise.0) as f64;
        let (f_mark, f_space, rate, bits) = match (&c.tones, &c.symbols) {
            (Some(tn), Some(sy)) => (Some(c.baseband.center.0 + tn.f_mark), Some(c.baseband.center.0 + tn.f_space), Some(sy.symbol_rate.0), Some(sy.bits.iter().map(|&b| if b { '1' } else { '0' }).collect::<String>())),
            (Some(tn), None) => (Some(c.baseband.center.0 + tn.f_mark), Some(c.baseband.center.0 + tn.f_space), None, None),
            _ => (None, None, None, None),
        };
        let g_center = ((center / 10e3).round() as i64) * 10_000;
        let g_bw = ((bw / 25e3).round() as i64).max(1) * 25;
        let g_dur = duration_class_ms(dur_ms);
        let tone_spacing = f_mark.zip(f_space).map(|(m, s)| m - s);
        let tx = self.conn.transaction()?;
        // upsert group, keeping the last ~8 sighting times for cadence estimation
        let existing: Option<(i64, String)> = tx
            .query_row("SELECT id, recent_times FROM groups WHERE receiver=?1 AND center_hz=?2 AND bandwidth_khz=?3 AND duration_class_ms=?4", params![c.receiver.0, g_center, g_bw, g_dur], |r| Ok((r.get(0)?, r.get(1)?)))
            .optional()?;
        let group_id = match existing {
            Some((id, recent)) => {
                let mut times: Vec<f64> = serde_json::from_str(&recent).unwrap_or_default();
                times.push(t);
                if times.len() > 8 {
                    times.remove(0);
                }
                tx.execute(
                    "UPDATE groups SET count=count+1, last_seen=?2, sum_snr=sum_snr+?3, fsk=max(fsk,?4), tone_spacing_hz=COALESCE(?5,tone_spacing_hz), symbol_rate=COALESCE(?6,symbol_rate), last_bits=COALESCE(?7,last_bits), recent_times=?8 WHERE id=?1",
                    params![id, t, snr, c.tones.is_some() as i64, tone_spacing, rate, bits, serde_json::to_string(&times)?],
                )?;
                id
            }
            None => {
                tx.execute(
                    "INSERT INTO groups (receiver, center_hz, bandwidth_khz, duration_class_ms, count, first_seen, last_seen, sum_snr, fsk, tone_spacing_hz, symbol_rate, last_bits, recent_times) VALUES (?1,?2,?3,?4,1,?5,?5,?6,?7,?8,?9,?10,?11)",
                    params![c.receiver.0, g_center, g_bw, g_dur, t, snr, c.tones.is_some() as i64, tone_spacing, rate, bits, serde_json::to_string(&vec![t])?],
                )?;
                tx.last_insert_rowid()
            }
        };
        // example IQ as cs8 (int8 I,Q), normalised so the burst's peak is ~0.9 of
        // full scale: 48 dB of range within a burst is plenty, and half the bytes.
        let peak = c.baseband.iq.iter().map(|s| s.re.abs().max(s.im.abs())).fold(1e-9f32, f32::max);
        let scale = 0.9 * 127.0 / peak;
        let mut iq = Vec::with_capacity(c.baseband.iq.len() * 2);
        for s in &c.baseband.iq {
            iq.push((s.re * scale).clamp(-128.0, 127.0) as i8 as u8);
            iq.push((s.im * scale).clamp(-128.0, 127.0) as i8 as u8);
        }
        let bytes = iq.len() as i64;
        tx.execute(
            "INSERT INTO chirps (group_id, receiver, time, center_hz, bandwidth_hz, duration_ms, peak_db, noise_db, snr_db, f_mark_hz, f_space_hz, symbol_rate, bits, sample_rate, samples, bytes, iq, family) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18)",
            params![group_id, c.receiver.0, t, center, bw, dur_ms, c.burst.peak.0 as f64, c.burst.noise.0 as f64, snr, f_mark, f_space, rate, bits, c.baseband.sample_rate.0 as i64, c.baseband.iq.len() as i64, bytes, iq, family],
        )?;
        self.total_bytes += bytes;
        // per-group cap: drop this group's oldest examples
        let freed: i64 = {
            let mut stmt = tx.prepare("SELECT id, bytes FROM chirps WHERE group_id=?1 ORDER BY id DESC LIMIT -1 OFFSET ?2")?;
            let old: Vec<(i64, i64)> = stmt.query_map(params![group_id, self.cfg.max_examples_per_group], |r| Ok((r.get(0)?, r.get(1)?)))?.collect::<std::result::Result<_, _>>()?;
            let mut freed = 0;
            for (id, b) in old {
                tx.execute("DELETE FROM chirps WHERE id=?1", params![id])?;
                freed += b;
            }
            freed
        };
        self.total_bytes -= freed;
        // per-family cap: a hopper's many groups share one budget
        let freed: i64 = {
            let mut stmt = tx.prepare("SELECT id, bytes FROM chirps WHERE family=?1 ORDER BY id DESC LIMIT -1 OFFSET ?2")?;
            let old: Vec<(i64, i64)> = stmt.query_map(params![family, self.cfg.max_examples_per_family], |r| Ok((r.get(0)?, r.get(1)?)))?.collect::<std::result::Result<_, _>>()?;
            let mut freed = 0;
            for (id, b) in old {
                tx.execute("DELETE FROM chirps WHERE id=?1", params![id])?;
                freed += b;
            }
            freed
        };
        self.total_bytes -= freed;
        // global cap: evict oldest overall
        while self.total_bytes > self.cfg.max_bytes {
            let oldest: Option<(i64, i64)> = tx.query_row("SELECT id, bytes FROM chirps ORDER BY id ASC LIMIT 1", [], |r| Ok((r.get(0)?, r.get(1)?))).optional()?;
            match oldest {
                Some((id, b)) => {
                    tx.execute("DELETE FROM chirps WHERE id=?1", params![id])?;
                    self.total_bytes -= b;
                }
                None => break,
            }
        }
        tx.commit()?;
        Ok(())
    }

    pub fn stats(&self) -> Result<StoreStats> {
        let (chirps, oldest, newest): (i64, Option<f64>, Option<f64>) = self.conn.query_row("SELECT COUNT(*), MIN(time), MAX(time) FROM chirps", [], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?;
        let groups: i64 = self.conn.query_row("SELECT COUNT(*) FROM groups", [], |r| r.get(0))?;
        Ok(StoreStats { chirps, groups, bytes: self.total_bytes, max_bytes: self.cfg.max_bytes, oldest, newest })
    }

    pub fn groups(&self, receiver: Option<&str>, min_count: i64) -> Result<Vec<GroupRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT g.id, g.receiver, g.center_hz, g.bandwidth_khz, g.duration_class_ms, g.count, g.first_seen, g.last_seen, g.sum_snr, g.fsk, g.tone_spacing_hz, g.symbol_rate, g.last_bits, g.recent_times,
                    (SELECT COUNT(*) FROM chirps c WHERE c.group_id=g.id) AS examples
             FROM groups g WHERE (?1 IS NULL OR g.receiver=?1) AND g.count >= ?2 ORDER BY g.count DESC",
        )?;
        let rows = stmt
            .query_map(params![receiver, min_count], |r| {
                let count: i64 = r.get(5)?;
                let sum_snr: f64 = r.get(8)?;
                let recent: String = r.get(13)?;
                let times: Vec<f64> = serde_json::from_str(&recent).unwrap_or_default();
                let mut gaps: Vec<f64> = times.windows(2).map(|w| w[1] - w[0]).filter(|g| *g > 0.05).collect();
                gaps.sort_by(|a, b| a.partial_cmp(b).unwrap());
                let period = if gaps.len() >= 2 { Some(gaps[gaps.len() / 2]) } else { None };
                Ok(GroupRow {
                    id: r.get(0)?,
                    receiver: r.get(1)?,
                    center_hz: r.get(2)?,
                    bandwidth_khz: r.get(3)?,
                    duration_class_ms: r.get(4)?,
                    count,
                    first_seen: r.get(6)?,
                    last_seen: r.get(7)?,
                    mean_snr_db: if count > 0 { sum_snr / count as f64 } else { 0.0 },
                    fsk: r.get::<_, i64>(9)? != 0,
                    tone_spacing_hz: r.get(10)?,
                    symbol_rate: r.get(11)?,
                    last_bits: r.get(12)?,
                    examples: r.get(14)?,
                    typical_period_s: period,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn list(&self, group_id: Option<i64>, receiver: Option<&str>, since: Option<f64>, limit: i64) -> Result<Vec<ChirpMeta>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, group_id, receiver, time, center_hz, bandwidth_hz, duration_ms, peak_db, noise_db, snr_db, f_mark_hz, f_space_hz, symbol_rate, bits, sample_rate, samples, bytes
             FROM chirps WHERE (?1 IS NULL OR group_id=?1) AND (?2 IS NULL OR receiver=?2) AND (?3 IS NULL OR time>=?3) ORDER BY id DESC LIMIT ?4",
        )?;
        let rows = stmt
            .query_map(params![group_id, receiver, since, limit], |r| {
                Ok(ChirpMeta { id: r.get(0)?, group_id: r.get(1)?, receiver: r.get(2)?, time: r.get(3)?, center_hz: r.get(4)?, bandwidth_hz: r.get(5)?, duration_ms: r.get(6)?, peak_db: r.get(7)?, noise_db: r.get(8)?, snr_db: r.get(9)?, f_mark_hz: r.get(10)?, f_space_hz: r.get(11)?, symbol_rate: r.get(12)?, bits: r.get(13)?, sample_rate: r.get(14)?, samples: r.get(15)?, bytes: r.get(16)? })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn get_iq(&self, id: i64) -> Result<Option<(ChirpMeta, Vec<u8>)>> {
        let meta = self.list(None, None, None, 1_000_000_000)?; // cheap enough: metadata only; refine if ever large
        let Some(m) = meta.into_iter().find(|m| m.id == id) else { return Ok(None) };
        let iq: Vec<u8> = self.conn.query_row("SELECT iq FROM chirps WHERE id=?1", params![id], |r| r.get(0))?;
        Ok(Some((m, iq)))
    }
}
