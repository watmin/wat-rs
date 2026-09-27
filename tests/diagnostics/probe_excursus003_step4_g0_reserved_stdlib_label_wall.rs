//! Excursus 003 step 4 (D4) — G0, the reserved-stdlib-label wall.
//!
//! BRIEF-envelope-step-4-the-primary-location-is-derived.md item 1's own STOP condition,
//! driven live rather than reasoned about: a user program CAN be loaded from a path
//! string identical to a stdlib file's own label (`wat/core.wat`) — a filesystem loader
//! always canonicalizes to an absolute path first (`FsLoader`/`ScopedLoader`), so the
//! collision reaches production only through a raw-label loader (`InMemoryLoader`) or a
//! real on-disk load that re-relativizes back to a stdlib's own bare path — but it IS
//! reachable, which makes a path-string-keyed "user source" record ambiguous UNLESS the
//! label itself is reserved. Builder ruling: refuse it, the same wall `:wat::` names get.
//!
//! This is the exact collision this excursus's own step-4 rider drove to justify the
//! STOP, promoted from a throwaway measurement into the permanent negative control.

use std::sync::Arc;
use wat::freeze::StartupError;
use wat::load::loader::{InMemoryLoader, LoadErrorKind, SourceLoader};

const ENTRY_LABEL: &str = "tests/diagnostics/probe_excursus003_step4_g0_reserved_stdlib_label_wall.wat";
const ENTRY_SRC: &str = include_str!("probe_excursus003_step4_g0_reserved_stdlib_label_wall.wat");
const COLLIDER_SRC: &str =
    include_str!("probe_excursus003_step4_g0_reserved_stdlib_label_wall__collider.wat");

#[test]
fn g0_a_user_load_at_a_stdlib_label_is_refused() {
    let mut mem = InMemoryLoader::new();
    mem.add_source("wat/core.wat", COLLIDER_SRC);
    let loader: Arc<dyn SourceLoader> = Arc::new(mem);

    let result = wat::freeze::startup_from_source(ENTRY_SRC, Some(ENTRY_LABEL), loader);

    match result {
        Err(StartupError::Load(e)) => {
            assert!(
                matches!(e.kind(), LoadErrorKind::ReservedStdlibLabel { label } if label == "wat/core.wat"),
                "expected LoadErrorKind::ReservedStdlibLabel {{ label: \"wat/core.wat\" }}; got {:?}",
                e.kind()
            );
        }
        other => panic!(
            "expected StartupError::Load(ReservedStdlibLabel) — a user load at a stdlib \
             label must be refused, not silently accepted; got {other:?}"
        ),
    }
}
