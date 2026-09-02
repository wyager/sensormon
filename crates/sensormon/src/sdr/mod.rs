//! In-process SDR drivers, bound at runtime with `dlopen` so one binary runs
//! anywhere the vendor library is installed (no link-time dependency, and the
//! cross-compiled binary needs no sysroot).

pub mod airspy;
pub mod rtlsdr;

use anyhow::{Context, Result};
use libloading::Library;

/// Load the first of several candidate library names.
pub fn load_library(names: &[&str]) -> Result<Library> {
    let mut last = None;
    for n in names {
        // SAFETY: loading a vendor library; its initialisers are what any C program runs.
        match unsafe { Library::new(n) } {
            Ok(l) => return Ok(l),
            Err(e) => last = Some(e),
        }
    }
    Err(anyhow::anyhow!("{:?}", last)).with_context(|| format!("none of {names:?} could be loaded"))
}
