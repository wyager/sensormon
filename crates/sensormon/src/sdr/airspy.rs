//! Airspy R2 / Mini via libairspy (dlopen'd).

use super::load_library;
use crate::config::AirspyGain;
use crate::source::{cs16_to_iq, BlockSink, IqSource, RunningSource};
use anyhow::{bail, Context, Result};
use libloading::{Library, Symbol};
use std::ffi::{c_int, c_void, CStr};
use std::sync::Arc;

#[repr(C)]
struct AirspyTransfer {
    device: *mut c_void,
    ctx: *mut c_void,
    samples: *mut c_void,
    sample_count: c_int,
    dropped_samples: u64,
    sample_type: c_int,
}

const SAMPLE_INT16_IQ: c_int = 2;

type CbFn = unsafe extern "C" fn(*mut AirspyTransfer) -> c_int;

/// Function table resolved from libairspy.
struct Api {
    _lib: Arc<Library>,
    init: unsafe extern "C" fn() -> c_int,
    open: unsafe extern "C" fn(*mut *mut c_void) -> c_int,
    open_sn: unsafe extern "C" fn(*mut *mut c_void, u64) -> c_int,
    close: unsafe extern "C" fn(*mut c_void) -> c_int,
    set_samplerate: unsafe extern "C" fn(*mut c_void, u32) -> c_int,
    set_sample_type: unsafe extern "C" fn(*mut c_void, c_int) -> c_int,
    set_freq: unsafe extern "C" fn(*mut c_void, u32) -> c_int,
    set_lna_gain: unsafe extern "C" fn(*mut c_void, u8) -> c_int,
    set_mixer_gain: unsafe extern "C" fn(*mut c_void, u8) -> c_int,
    set_vga_gain: unsafe extern "C" fn(*mut c_void, u8) -> c_int,
    set_lna_agc: unsafe extern "C" fn(*mut c_void, u8) -> c_int,
    set_mixer_agc: unsafe extern "C" fn(*mut c_void, u8) -> c_int,
    set_linearity_gain: unsafe extern "C" fn(*mut c_void, u8) -> c_int,
    set_sensitivity_gain: unsafe extern "C" fn(*mut c_void, u8) -> c_int,
    set_rf_bias: unsafe extern "C" fn(*mut c_void, u8) -> c_int,
    set_packing: unsafe extern "C" fn(*mut c_void, u8) -> c_int,
    start_rx: unsafe extern "C" fn(*mut c_void, CbFn, *mut c_void) -> c_int,
    stop_rx: unsafe extern "C" fn(*mut c_void) -> c_int,
    error_name: unsafe extern "C" fn(c_int) -> *const std::ffi::c_char,
}

macro_rules! sym {
    ($lib:expr, $name:literal) => {{
        // SAFETY: symbol signatures transcribed from libairspy/airspy.h (1.0.11).
        let s: Symbol<_> = unsafe { $lib.get($name) }.with_context(|| format!("libairspy lacks {}", std::str::from_utf8($name).unwrap()))?;
        *s
    }};
}

impl Api {
    fn load() -> Result<Self> {
        let lib = Arc::new(load_library(&["libairspy.so.0", "libairspy.so", "libairspy.dylib", "libairspy.0.dylib"])?);
        Ok(Api {
            init: sym!(lib, b"airspy_init\0"),
            open: sym!(lib, b"airspy_open\0"),
            open_sn: sym!(lib, b"airspy_open_sn\0"),
            close: sym!(lib, b"airspy_close\0"),
            set_samplerate: sym!(lib, b"airspy_set_samplerate\0"),
            set_sample_type: sym!(lib, b"airspy_set_sample_type\0"),
            set_freq: sym!(lib, b"airspy_set_freq\0"),
            set_lna_gain: sym!(lib, b"airspy_set_lna_gain\0"),
            set_mixer_gain: sym!(lib, b"airspy_set_mixer_gain\0"),
            set_vga_gain: sym!(lib, b"airspy_set_vga_gain\0"),
            set_lna_agc: sym!(lib, b"airspy_set_lna_agc\0"),
            set_mixer_agc: sym!(lib, b"airspy_set_mixer_agc\0"),
            set_linearity_gain: sym!(lib, b"airspy_set_linearity_gain\0"),
            set_sensitivity_gain: sym!(lib, b"airspy_set_sensitivity_gain\0"),
            set_rf_bias: sym!(lib, b"airspy_set_rf_bias\0"),
            set_packing: sym!(lib, b"airspy_set_packing\0"),
            start_rx: sym!(lib, b"airspy_start_rx\0"),
            stop_rx: sym!(lib, b"airspy_stop_rx\0"),
            error_name: sym!(lib, b"airspy_error_name\0"),
            _lib: lib,
        })
    }

    fn check(&self, what: &str, rc: c_int) -> Result<()> {
        if rc == 0 {
            return Ok(());
        }
        // SAFETY: error_name returns a static string for any code.
        let name = unsafe { CStr::from_ptr((self.error_name)(rc)) }.to_string_lossy().into_owned();
        bail!("{what}: {name} ({rc})")
    }
}

pub struct AirspySource {
    pub serial: Option<u64>,
    pub center_hz: u32,
    pub sample_rate: u32,
    pub gain: AirspyGain,
    pub bias_tee: bool,
}

/// State handed to the C callback (boxed, address-stable).
struct CbState {
    sink: BlockSink,
}

unsafe extern "C" fn on_transfer(t: *mut AirspyTransfer) -> c_int {
    // SAFETY: libairspy passes a valid transfer whose ctx is our Box<CbState>,
    // and only calls us from its single streaming thread.
    let t = unsafe { &*t };
    let state = unsafe { &mut *(t.ctx as *mut CbState) };
    if t.sample_type == SAMPLE_INT16_IQ && t.sample_count > 0 {
        let raw = unsafe { std::slice::from_raw_parts(t.samples as *const i16, t.sample_count as usize * 2) };
        state.sink.push(cs16_to_iq(raw), t.dropped_samples);
    }
    0
}

struct Running {
    api: Api,
    dev: *mut c_void,
    state: *mut CbState,
}
// SAFETY: the device handle is only used from the thread that stops it.
unsafe impl Send for Running {}

impl RunningSource for Running {
    fn stop(self: Box<Self>) {
        // SAFETY: stop_rx joins the streaming thread before returning, after which the callback state can be freed.
        unsafe {
            (self.api.stop_rx)(self.dev);
            (self.api.close)(self.dev);
            drop(Box::from_raw(self.state));
        }
    }
}

impl IqSource for AirspySource {
    fn describe(&self) -> String {
        format!("airspy serial={:?} {} Hz @ {} S/s", self.serial.map(|s| format!("{s:016x}")), self.center_hz, self.sample_rate)
    }

    fn start(self: Box<Self>, sink: BlockSink) -> Result<Box<dyn RunningSource>> {
        let api = Api::load()?;
        let mut dev: *mut c_void = std::ptr::null_mut();
        // SAFETY: plain FFI calls with valid arguments; `dev` is an out-pointer.
        unsafe {
            api.check("airspy_init", (api.init)())?;
            match self.serial {
                Some(sn) => api.check("airspy_open_sn", (api.open_sn)(&mut dev, sn))?,
                None => api.check("airspy_open", (api.open)(&mut dev))?,
            }
            api.check("set_sample_type", (api.set_sample_type)(dev, SAMPLE_INT16_IQ))?;
            api.check("set_packing", (api.set_packing)(dev, 0))?;
            api.check("set_samplerate", (api.set_samplerate)(dev, self.sample_rate))?;
            api.check("set_freq", (api.set_freq)(dev, self.center_hz))?;
            match self.gain {
                AirspyGain::Stages { lna, mix, vga } => {
                    api.check("set_lna_agc", (api.set_lna_agc)(dev, 0))?;
                    api.check("set_mixer_agc", (api.set_mixer_agc)(dev, 0))?;
                    api.check("set_lna_gain", (api.set_lna_gain)(dev, lna.min(15)))?;
                    api.check("set_mixer_gain", (api.set_mixer_gain)(dev, mix.min(15)))?;
                    api.check("set_vga_gain", (api.set_vga_gain)(dev, vga.min(15)))?;
                }
                AirspyGain::Preset { linearity: Some(g), .. } => api.check("set_linearity_gain", (api.set_linearity_gain)(dev, g.min(21)))?,
                AirspyGain::Preset { sensitivity: Some(g), .. } => api.check("set_sensitivity_gain", (api.set_sensitivity_gain)(dev, g.min(21)))?,
                AirspyGain::Preset { .. } => bail!("airspy gain: give lna/mix/vga, linearity, or sensitivity"),
            }
            api.check("set_rf_bias", (api.set_rf_bias)(dev, self.bias_tee as u8))?;
        }
        let state = Box::into_raw(Box::new(CbState { sink }));
        // SAFETY: state outlives streaming (freed in stop after stop_rx).
        let rc = unsafe { (api.start_rx)(dev, on_transfer, state as *mut c_void) };
        if let Err(e) = api.check("airspy_start_rx", rc) {
            unsafe {
                (api.close)(dev);
                drop(Box::from_raw(state));
            }
            return Err(e);
        }
        Ok(Box::new(Running { api, dev, state }))
    }
}
