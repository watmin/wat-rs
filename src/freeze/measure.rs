//! Excursus 003 D5 — opt-in freeze RE-FREEZE counting.
//!
//! `AUDIT-the-shape-of-an-error.md`'s Strike B worklist (the S2 cost row) asks not only
//! where one freeze spends its time but *how many times* the stdlib gets frozen across a
//! run — `startup_beside` / `call_beside_value` and their kin each build a brand-new
//! [`super::FrozenWorld`] from scratch, and nothing in `build_env` (`src/freeze/env.rs`)
//! reuses a prior freeze's registries (the one exception, [`super::env::stdlib_snapshot`],
//! is a `OnceLock` used ONLY by two test/reflection call sites — `src/check.rs` and
//! `src/reflect/verbs.rs` — never by the production `startup_from_forms_post_config` path).
//!
//! [`log_freeze_if_enabled`] is called once per successful [`super::FrozenWorld::freeze`] —
//! the one door every startup path funnels through, in-process or in a spawned child. When
//! `WAT_FREEZE_COUNT_LOG` names a path, it appends one line (this process's pid) to that
//! file; `wc -l` the file after a run (e.g. a full `scripts/floor.sh`, or one nextest
//! binary) for the exact count of successful freezes that ran with the var set, across
//! every process. `cargo nextest` isolates each test in its own process, so this is the
//! only way to get a SUITE-WIDE total rather than a per-process one.
//!
//! Zero cost when unset: one `OnceLock` read, no `open`, no `write`. Never touches stdout
//! or stderr under any condition — a file path only — so it cannot be the "non-EDN on the
//! wire" hazard the harness rules warn about. Never panics: a failed open/write is
//! swallowed, because a measurement instrument must not be the reason a real freeze fails.

use std::io::Write;
use std::path::PathBuf;
use std::sync::OnceLock;

fn count_log_path() -> Option<&'static PathBuf> {
    static PATH: OnceLock<Option<PathBuf>> = OnceLock::new();
    PATH.get_or_init(|| std::env::var_os("WAT_FREEZE_COUNT_LOG").map(PathBuf::from))
        .as_ref()
}

/// Called once per successful [`super::FrozenWorld::freeze`]. See the module doc.
pub(crate) fn log_freeze_if_enabled() {
    let Some(path) = count_log_path() else {
        return;
    };
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
        // One short line, O_APPEND: on Linux, a write this small is atomic with respect to
        // other appenders (including other nextest-isolated processes), so lines never
        // interleave. The pid is the payload only because SOME payload is useful for a
        // human skimming the file; the count that matters is the LINE count.
        let _ = writeln!(f, "{}", std::process::id());
    }
}
