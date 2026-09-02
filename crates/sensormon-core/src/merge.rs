//! Cross-receiver merging: the same transmission decoded by several receivers
//! becomes one `Event` listing every receiver's signal stats.

use crate::event::{Event, Reception, ReceiverEvent};
use chrono::{DateTime, Duration, Utc};
use std::collections::BTreeMap;

struct Pending {
    first: ReceiverEvent,
    heard_by: BTreeMap<crate::event::ReceiverId, crate::event::Signal>,
}

/// Holds each transmission for `window` after its first copy, collecting
/// other receivers' copies (matched by identical raw bytes), then emits.
pub struct Merger {
    window: Duration,
    pending: Vec<Pending>,
}

impl Merger {
    pub fn new(window: Duration) -> Self {
        Merger { window, pending: Vec::new() }
    }

    /// Accept one receiver's decode; returns any events whose window has closed.
    pub fn push(&mut self, ev: ReceiverEvent, now: DateTime<Utc>) -> Vec<Event> {
        match self.pending.iter_mut().find(|p| p.first.raw == ev.raw && ev.time - p.first.time <= self.window) {
            Some(p) => {
                p.heard_by.insert(ev.receiver.clone(), ev.signal);
            }
            None => {
                let mut heard_by = BTreeMap::new();
                heard_by.insert(ev.receiver.clone(), ev.signal);
                self.pending.push(Pending { first: ev, heard_by });
            }
        }
        self.flush(now)
    }

    /// Emit everything whose window has closed as of `now`.
    pub fn flush(&mut self, now: DateTime<Utc>) -> Vec<Event> {
        let window = self.window;
        let (done, keep): (Vec<_>, Vec<_>) = self.pending.drain(..).partition(|p| now - p.first.time > window);
        self.pending = keep;
        done.into_iter().map(|p| Event { reception: Reception { time: p.first.time, heard_by: p.heard_by }, sensor: p.first.sensor, raw: p.first.raw }).collect()
    }

    /// Emit everything regardless of window (shutdown).
    pub fn drain(&mut self) -> Vec<Event> {
        self.flush(DateTime::<Utc>::MAX_UTC)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{Payload, ReceiverId, Signal, Wh51};
    use crate::units::{Db, Hertz};

    fn ev(rx: &str, t: DateTime<Utc>, raw: &[u8]) -> ReceiverEvent {
        let sig = Signal { center: Hertz(915e6), f_mark: Hertz(915.03e6), f_space: Hertz(914.96e6), symbol_rate: Hertz(17240.0), rssi: Db(-30.0), snr: Db(20.0), noise: Db(-50.0) };
        ReceiverEvent { receiver: ReceiverId(rx.into()), time: t, signal: sig, sensor: Payload::Wh51(Wh51 { id: 1, battery_mv: 1500, battery_level: 0.9, moisture_pct: 30, boost: 0, ad_raw: 100 }), raw: raw.to_vec() }
    }

    #[test]
    fn merges_same_raw_within_window() {
        let t0 = Utc::now();
        let mut m = Merger::new(Duration::seconds(2));
        assert!(m.push(ev("a", t0, &[1, 2, 3]), t0).is_empty());
        assert!(m.push(ev("b", t0 + Duration::milliseconds(300), &[1, 2, 3]), t0 + Duration::milliseconds(300)).is_empty());
        let out = m.flush(t0 + Duration::seconds(3));
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].reception.heard_by.len(), 2);
        assert_eq!(out[0].reception.time, t0);
    }

    #[test]
    fn different_raw_are_separate() {
        let t0 = Utc::now();
        let mut m = Merger::new(Duration::seconds(2));
        m.push(ev("a", t0, &[1]), t0);
        m.push(ev("a", t0, &[2]), t0);
        assert_eq!(m.drain().len(), 2);
    }
}
