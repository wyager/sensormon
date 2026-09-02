//! RTL2832U dongles via librtlsdr (dlopen'd). `rtlsdr_read_async` blocks, so it
//! runs on its own thread.

use super::load_library;
use crate::config::RtlsdrGain;
use crate::source::{cu8_to_iq, BlockSink, IqSource, RunningSource};
use anyhow::{bail, Context, Result};
use libloading::{Library, Symbol};
use std::ffi::{c_int, c_void};
use std::sync::Arc;
use std::thread::JoinHandle;

type CbFn = unsafe extern "C" fn(*mut u8, u32, *mut c_void);

#[derive(Clone)]
struct Api {
    _lib: Arc<Library>,
    open: unsafe extern "C" fn(*mut *mut c_void, u32) -> c_int,
    close: unsafe extern "C" fn(*mut c_void) -> c_int,
    set_sample_rate: unsafe extern "C" fn(*mut c_void, u32) -> c_int,
    set_center_freq: unsafe extern "C" fn(*mut c_void, u32) -> c_int,
    set_tuner_gain_mode: unsafe extern "C" fn(*mut c_void, c_int) -> c_int,
    set_tuner_gain: unsafe extern "C" fn(*mut c_void, c_int) -> c_int,
    set_agc_mode: unsafe extern "C" fn(*mut c_void, c_int) -> c_int,
    set_bias_tee: unsafe extern "C" fn(*mut c_void, c_int) -> c_int,
    reset_buffer: unsafe extern "C" fn(*mut c_void) -> c_int,
    read_async: unsafe extern "C" fn(*mut c_void, CbFn, *mut c_void, u32, u32) -> c_int,
    cancel_async: unsafe extern "C" fn(*mut c_void) -> c_int,
}

macro_rules! sym {
    ($lib:expr, $name:literal) => {{
        // SAFETY: signatures transcribed from rtl-sdr.h (librtlsdr 2.0.1).
        let s: Symbol<_> = unsafe { $lib.get($name) }.with_context(|| format!("librtlsdr lacks {}", std::str::from_utf8($name).unwrap()))?;
        *s
    }};
}

impl Api {
    fn load() -> Result<Self> {
        let lib = Arc::new(load_library(&["librtlsdr.so.0", "librtlsdr.so.2", "librtlsdr.so", "librtlsdr.dylib", "librtlsdr.0.dylib"])?);
        Ok(Api {
            open: sym!(lib, b"rtlsdr_open\0"),
            close: sym!(lib, b"rtlsdr_close\0"),
            set_sample_rate: sym!(lib, b"rtlsdr_set_sample_rate\0"),
            set_center_freq: sym!(lib, b"rtlsdr_set_center_freq\0"),
            set_tuner_gain_mode: sym!(lib, b"rtlsdr_set_tuner_gain_mode\0"),
            set_tuner_gain: sym!(lib, b"rtlsdr_set_tuner_gain\0"),
            set_agc_mode: sym!(lib, b"rtlsdr_set_agc_mode\0"),
            set_bias_tee: sym!(lib, b"rtlsdr_set_bias_tee\0"),
            reset_buffer: sym!(lib, b"rtlsdr_reset_buffer\0"),
            read_async: sym!(lib, b"rtlsdr_read_async\0"),
            cancel_async: sym!(lib, b"rtlsdr_cancel_async\0"),
            _lib: lib,
        })
    }

    fn check(what: &str, rc: c_int) -> Result<()> {
        if rc < 0 {
            bail!("{what}: rc {rc}");
        }
        Ok(())
    }
}

pub struct RtlsdrSource {
    pub index: u32,
    /// Centers to cycle through; a single entry means no hopping.
    pub centers_hz: Vec<u32>,
    pub dwell: std::time::Duration,
    pub sample_rate: u32,
    pub gain: RtlsdrGain,
    pub bias_tee: bool,
}

struct CbState {
    sink: BlockSink,
}

unsafe extern "C" fn on_block(buf: *mut u8, len: u32, ctx: *mut c_void) {
    // SAFETY: librtlsdr hands us a valid buffer; ctx is our Box<CbState>.
    let state = unsafe { &mut *(ctx as *mut CbState) };
    let raw = unsafe { std::slice::from_raw_parts(buf, len as usize) };
    state.sink.push(cu8_to_iq(raw), 0);
}

struct Running {
    api: Api,
    dev: DevPtr,
    thread: Option<JoinHandle<()>>,
    hopper: Option<(Arc<std::sync::atomic::AtomicBool>, JoinHandle<()>)>,
    state: *mut CbState,
}

#[derive(Clone, Copy)]
struct DevPtr(*mut c_void);
// SAFETY: librtlsdr's device handle may be used from another thread to cancel/close.
unsafe impl Send for DevPtr {}
unsafe impl Send for Running {}

impl RunningSource for Running {
    fn stop(mut self: Box<Self>) {
        if let Some((flag, t)) = self.hopper.take() {
            flag.store(true, std::sync::atomic::Ordering::Relaxed);
            let _ = t.join();
        }
        // SAFETY: cancel_async makes read_async return on its thread; then close and free.
        unsafe {
            (self.api.cancel_async)(self.dev.0);
        }
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
        unsafe {
            (self.api.close)(self.dev.0);
            drop(Box::from_raw(self.state));
        }
    }
}

impl IqSource for RtlsdrSource {
    fn describe(&self) -> String {
        format!("rtlsdr index={} centers={:?} Hz dwell={:?} @ {} S/s", self.index, self.centers_hz, self.dwell, self.sample_rate)
    }

    fn start(self: Box<Self>, sink: BlockSink) -> Result<Box<dyn RunningSource>> {
        let api = Api::load()?;
        let mut dev: *mut c_void = std::ptr::null_mut();
        // SAFETY: plain FFI with valid arguments.
        unsafe {
            Api::check("rtlsdr_open", (api.open)(&mut dev, self.index))?;
            Api::check("set_sample_rate", (api.set_sample_rate)(dev, self.sample_rate))?;
            Api::check("set_center_freq", (api.set_center_freq)(dev, self.centers_hz[0]))?;
            match self.gain {
                RtlsdrGain::Auto(_) => {
                    Api::check("set_tuner_gain_mode", (api.set_tuner_gain_mode)(dev, 0))?;
                    Api::check("set_agc_mode", (api.set_agc_mode)(dev, 1))?;
                }
                RtlsdrGain::Db(db) => {
                    Api::check("set_tuner_gain_mode", (api.set_tuner_gain_mode)(dev, 1))?;
                    Api::check("set_agc_mode", (api.set_agc_mode)(dev, 0))?;
                    Api::check("set_tuner_gain", (api.set_tuner_gain)(dev, (db * 10.0).round() as c_int))?;
                }
            }
            let _ = (api.set_bias_tee)(dev, self.bias_tee as c_int); // absent on old builds; not fatal
            Api::check("reset_buffer", (api.reset_buffer)(dev))?;
        }
        let center_handle = sink.center_handle();
        center_handle.store((self.centers_hz[0] as f64).to_bits(), std::sync::atomic::Ordering::Relaxed);
        let state = Box::into_raw(Box::new(CbState { sink }));
        let dev = DevPtr(dev);
        let api2 = api.clone();
        let state_ptr = StatePtr(state);
        // Frequency hopper: retune every `dwell` and publish the new center so
        // blocks are tagged (the runtime discards a settle window after each hop).
        let hopper = if self.centers_hz.len() > 1 {
            let flag = Arc::new(std::sync::atomic::AtomicBool::new(false));
            let flag2 = flag.clone();
            let api3 = api.clone();
            let centers = self.centers_hz.clone();
            let dwell = self.dwell;
            let t = std::thread::Builder::new().name("rtlsdr-hop".into()).spawn(move || {
                let dev = dev;
                let mut i = 0usize;
                loop {
                    let deadline = std::time::Instant::now() + dwell;
                    while std::time::Instant::now() < deadline {
                        if flag2.load(std::sync::atomic::Ordering::Relaxed) {
                            return;
                        }
                        std::thread::sleep(std::time::Duration::from_millis(50));
                    }
                    i = (i + 1) % centers.len();
                    // SAFETY: librtlsdr permits retuning while read_async runs (rtl_433 hops this way).
                    let rc = unsafe { (api3.set_center_freq)(dev.0, centers[i]) };
                    if rc == 0 {
                        center_handle.store((centers[i] as f64).to_bits(), std::sync::atomic::Ordering::Relaxed);
                        eprintln!("rtlsdr: retuned to {} Hz", centers[i]);
                    } else {
                        eprintln!("rtlsdr: retune to {} failed ({rc})", centers[i]);
                    }
                }
            })?;
            Some((flag, t))
        } else {
            None
        };
        let thread = std::thread::Builder::new().name("rtlsdr-usb".into()).spawn(move || {
            // move the whole Send wrappers in (edition-2021 closures would otherwise capture just the raw pointer fields)
            let (dev, state_ptr) = (dev, state_ptr);
            // SAFETY: blocks until cancel_async; state outlives it (freed after join).
            let rc = unsafe { (api2.read_async)(dev.0, on_block, state_ptr.0 as *mut c_void, 0, 0) };
            if rc != 0 {
                eprintln!("rtlsdr_read_async returned {rc}");
            }
        })?;
        Ok(Box::new(Running { api, dev, thread: Some(thread), hopper, state }))
    }
}

#[derive(Clone, Copy)]
struct StatePtr(*mut CbState);
unsafe impl Send for StatePtr {}
