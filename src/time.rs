//! Process boot clock — arc 056/259.
//!
//! The `:wat::time::*` dispatch surface (Instant/Duration construction,
//! arithmetic, ISO 8601 round-trips, `now`/`ago`/`from-now`) moved to
//! `src/intrinsic/time.rs` as `#[wat_intrinsic]`-registered handlers
//! (arc 255.1c-time, home #2). What's left here is Rust-internal API with
//! no `:wat::` FQDN of its own — `process_boot_instant` / `set_process_boot_instant`,
//! consumed by `freeze.rs`, `kernel/spawn.rs`, and `distribution/mod.rs` to
//! measure a process's own boot instant (pid-keyed, fork-safe).

use chrono::{DateTime, Utc};

use std::sync::Mutex;

// ─── Process-level boot clock ─────────────────────────────────────────
//
// pid-keyed and fork-safe: captures `now` lazily on first call; re-captures
// across a fork (stored pid != current pid) so a forked `:process` peer
// measures its OWN boot, not the parent's inherited value.
//
// Both fns are `pub` so the test crate can reach them as `wat::time::*`.

static PROCESS_BOOT: Mutex<Option<(u32, DateTime<Utc>)>> = Mutex::new(None);

/// This process's boot instant. Captured lazily on first call; re-captured
/// across a fork (pid change) so a `:process` peer measures its own boot.
/// pid-keyed.
pub fn process_boot_instant() -> DateTime<Utc> {
    let pid = std::process::id();
    let mut g = PROCESS_BOOT.lock().unwrap_or_else(|e| e.into_inner());
    match *g {
        Some((p, inst)) if p == pid => inst,
        _ => {
            let now = Utc::now();
            *g = Some((pid, now));
            now
        }
    }
}

/// Explicitly set this process's boot instant (wat-cli at its earliest point;
/// tests inject a known value for deterministic timing). pid-keyed to the caller.
pub fn set_process_boot_instant(inst: DateTime<Utc>) {
    let pid = std::process::id();
    *PROCESS_BOOT.lock().unwrap_or_else(|e| e.into_inner()) = Some((pid, inst));
}
