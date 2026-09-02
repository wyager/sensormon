//! Raw IQ file reading (offline decoding and tests).

use anyhow::Result;
use sensormon_core::dsp::Iq;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

#[derive(Clone, Copy)]
pub enum Format {
    Cu8,
    Cs16,
}

pub struct Reader {
    inner: BufReader<File>,
    format: Format,
    raw: Vec<u8>,
}

impl Reader {
    pub fn open(path: &Path, format: Format) -> Result<Self> {
        Ok(Reader { inner: BufReader::with_capacity(1 << 20, File::open(path)?), format, raw: Vec::new() })
    }

    /// Read up to `n` complex samples into `out` (cleared first). Returns how many.
    pub fn read_block(&mut self, out: &mut Vec<Iq>, n: usize) -> Result<usize> {
        let bytes_per = match self.format {
            Format::Cu8 => 2,
            Format::Cs16 => 4,
        };
        self.raw.resize(n * bytes_per, 0);
        let mut got = 0;
        while got < self.raw.len() {
            let r = self.inner.read(&mut self.raw[got..])?;
            if r == 0 {
                break;
            }
            got += r;
        }
        let samples = got / bytes_per;
        out.clear();
        match self.format {
            Format::Cu8 => out.extend(self.raw[..samples * 2].chunks_exact(2).map(|c| Iq::new((c[0] as f32 - 127.5) / 127.5, (c[1] as f32 - 127.5) / 127.5))),
            Format::Cs16 => out.extend(self.raw[..samples * 4].chunks_exact(4).map(|c| Iq::new(i16::from_le_bytes([c[0], c[1]]) as f32 / 32768.0, i16::from_le_bytes([c[2], c[3]]) as f32 / 32768.0))),
        }
        Ok(samples)
    }
}
