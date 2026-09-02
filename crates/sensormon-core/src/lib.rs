//! sensormon-core: everything between IQ samples and typed sensor events, as
//! pure functions and explicit state structs. No I/O, no threads, no globals.
//!
//! Pipeline (per receiver):
//! `dsp::burst::BurstDetector` → `dsp::extract::extract` → `dsp::fsk::demod_fsk`
//! → `protocol::decode_all` → `event::ReceiverEvent`; then `merge::Merger`
//! joins receivers into `event::Event`.

pub mod bits;
pub mod crc;
pub mod dsp;
pub mod event;
pub mod merge;
pub mod pipeline;
pub mod protocol;
pub mod units;

pub use event::{Event, Payload, ReceiverEvent, ReceiverId, Reception, Signal};
pub use units::{Db, Hertz, SampleIndex, SampleRate};
