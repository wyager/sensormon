//! sensormon binary: runtime around `sensormon-core`.

mod chirps;
mod config;
mod filesource;
mod http;
mod iqfile;
mod runtime;
mod sdr;
mod source;

use anyhow::{Context, Result};
use chrono::Utc;
use clap::{Parser, Subcommand, ValueEnum};
use sensormon_core::pipeline::{Pipeline, PipelineConfig};
use sensormon_core::{Hertz, ReceiverId, SampleRate};
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "sensormon", about = "Wideband multi-signal ISM sensor receiver")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Clone, Copy, ValueEnum)]
enum Format {
    /// unsigned 8-bit I/Q (rtl_sdr, rtl_433 .cu8)
    Cu8,
    /// signed 16-bit I/Q (airspy_rx -t 2, rtl_433 .cs16)
    Cs16,
}

#[derive(Subcommand)]
enum Cmd {
    /// Run receivers from a config file and serve events over HTTP.
    Run {
        #[arg(long, default_value = "sensormon.toml")]
        config: PathBuf,
    },
    /// Run the full pipeline over a recorded IQ file and print decoded events as JSON lines.
    DecodeFile {
        file: PathBuf,
        #[arg(long)]
        rate: u32,
        /// Center frequency in Hz.
        #[arg(long)]
        center: f64,
        #[arg(long, value_enum)]
        format: Format,
        #[arg(long, default_value = "file")]
        receiver: String,
        /// Print per-sensor counts instead of every event.
        #[arg(long)]
        summary: bool,
        /// Samples per block fed to the pipeline.
        #[arg(long, default_value_t = 65536)]
        block: usize,
        /// Print per-burst diagnostics (stderr) for bursts inside [from, to] seconds.
        #[arg(long)]
        trace: bool,
        #[arg(long, default_value_t = 0.0)]
        from: f64,
        #[arg(long, default_value_t = f64::MAX)]
        to: f64,
        /// Also record every undecoded burst into this chirp database (see /chirps).
        #[arg(long)]
        chirps_db: Option<PathBuf>,
    },
}

fn main() -> Result<()> {
    match Cli::parse().cmd {
        Cmd::Run { config } => run(&config),
        Cmd::DecodeFile { file, rate, center, format, receiver, summary, block, trace, from, to, chirps_db } => {
            let rate = SampleRate(rate);
            // file mode: event time = offset into the file, from the Unix epoch
            let epoch = chrono::DateTime::<Utc>::from_timestamp(0, 0).unwrap();
            let mut pipeline = Pipeline::new(ReceiverId(receiver), rate, Hertz(center), PipelineConfig::default(), epoch);
            pipeline.set_trace(trace);
            let mut store = match &chirps_db {
                Some(path) => {
                    pipeline.set_keep_undecoded(true);
                    Some(chirps::ChirpStore::open(chirps::ChirpStoreConfig { path: path.display().to_string(), max_bytes: 50_000_000, max_examples_per_group: 6, receivers: Vec::new() })?)
                }
                None => None,
            };
            let mut reader = iqfile::Reader::open(&file, match format { Format::Cu8 => iqfile::Format::Cu8, Format::Cs16 => iqfile::Format::Cs16 }).with_context(|| format!("open {}", file.display()))?;
            let mut buf = Vec::with_capacity(block);
            let mut counts: BTreeMap<String, usize> = BTreeMap::new();
            let t_start = std::time::Instant::now();
            while reader.read_block(&mut buf, block)? > 0 {
                let events = pipeline.push(&buf);
                if let Some(st) = store.as_mut() {
                    for c in pipeline.take_chirps() {
                        st.insert(&c)?;
                    }
                }
                for t in pipeline.take_traces() {
                    let t0 = rate.seconds_of(t.burst.start.0);
                    if t0 < from || t0 > to {
                        continue;
                    }
                    let ms = rate.seconds_of(t.burst.start.distance_to(t.burst.end)) * 1e3;
                    let sym = t.symbols.as_ref().map(|s| format!("tones {:+.0}/{:+.0} kHz rate {:.0} snr {:.0} rssi {:.0} bits {}", (s.f_space.0 - center) / 1e3, (s.f_mark.0 - center) / 1e3, s.symbol_rate.0, s.snr.0, s.rssi.0, s.bits.len())).unwrap_or_else(|| "no-tones".into());
                    eprintln!("t={t0:8.3}s {ms:6.1}ms  {:+7.0}kHz bw {:5.0}k peak {:5.1} noise {:5.1} | {sym} | syncs {} frames {}", (t.burst.center().0 - center) / 1e3, t.burst.bandwidth().0 / 1e3, t.burst.peak.0, t.burst.noise.0, t.syncs, t.frames);
                    if std::env::var_os("SENSORMON_TRACE_BITS").is_some() {
                        if let Some(s) = &t.symbols {
                            eprintln!("    bits: {}", s.bits.iter().map(|&b| if b { '1' } else { '0' }).collect::<String>());
                        }
                    }
                }
                for ev in events {
                    if summary {
                        *counts.entry(format!("{} {:06x}", ev.sensor.model(), ev.sensor.sensor_id())).or_default() += 1;
                    } else {
                        println!("{}", serde_json::to_string(&ev)?);
                    }
                }
            }
            let s = pipeline.stats();
            let secs = rate.seconds_of(s.samples);
            eprintln!("{:.1}s of samples in {:.1}s wall; bursts={} demodulated={} frames={} lost_from_ring={}", secs, t_start.elapsed().as_secs_f64(), s.bursts, s.demodulated, s.decoded_frames, s.bursts_lost_from_ring);
            eprintln!("stage time: detect {:.2}s  extract {:.2}s  demod {:.2}s  decode {:.2}s", s.ns_detect as f64 / 1e9, s.ns_extract as f64 / 1e9, s.ns_demod as f64 / 1e9, s.ns_decode as f64 / 1e9);
            for (k, v) in counts {
                println!("{v:4}  {k}");
            }
            if let Some(st) = &store {
                let cs = st.stats()?;
                eprintln!("chirps: {} stored in {} groups, {:.1} MB", cs.chirps, cs.groups, cs.bytes as f64 / 1e6);
            }
            Ok(())
        }
    }
}

fn run(config_path: &std::path::Path) -> Result<()> {
    let cfg = config::load(config_path)?;
    let rt = tokio::runtime::Builder::new_multi_thread().enable_all().build()?;
    rt.block_on(async {
        let runtime = runtime::Runtime::start(&cfg)?;
        let events = runtime.events.clone();
        let chirps = runtime.chirps.clone();
        let runtime = std::sync::Arc::new(std::sync::Mutex::new(runtime));
        let app = http::router(http::AppState { runtime: runtime.clone(), events: events.clone(), chirps });
        let listener = tokio::net::TcpListener::bind(&cfg.http.bind).await.with_context(|| format!("bind {}", cfg.http.bind))?;
        eprintln!("http: listening on {}", cfg.http.bind);
        // log events to stderr as well, so `journalctl` shows decodes
        let mut log_rx = events.subscribe();
        tokio::spawn(async move {
            while let Ok(ev) = log_rx.recv().await {
                let rx: Vec<String> = ev.reception.heard_by.iter().map(|(r, s)| format!("{r}@{:.3}M:snr{:.0}", s.center.0 / 1e6, s.snr.0)).collect();
                eprintln!("{} {} {:06x} [{}]", ev.reception.time.format("%H:%M:%S%.3f"), ev.sensor.model(), ev.sensor.sensor_id(), rx.join(","));
            }
        });
        let serve = axum::serve(listener, app);
        tokio::select! {
            r = serve => { r?; }
            _ = tokio::signal::ctrl_c() => { eprintln!("stopping"); }
        }
        runtime.lock().unwrap().stop();
        Ok::<(), anyhow::Error>(())
    })
}
