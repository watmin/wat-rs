//! the-inert-clause-is-refused — `:deadline-ms` in `:satisfies` mode is a
//! compile-time `macro-error`. The clause governs nothing (measured: 10000 vs
//! 300, both directions). Probe 1's shape is the fixture; it must never run.

use wat::freeze::{startup_from_file, StartupError};
use wat::macros::{MacroError, MacroErrorKind};

#[test]
fn satisfies_deadline_ms_is_refused_at_compile_time() {
    let r = startup_from_file("tests/services/probe_inert_clause_is_refused.wat");
    wat::assert_startup_error!(r,
        StartupError::Macro(MacroError {
            kind: MacroErrorKind::ProgramBodyEvalFailed { macro_name, cause },
            ..
        }) if macro_name == ":wat::service::defservice"
            && matches!(
                &cause.kind,
                MacroErrorKind::MalformedTemplate { reason }
                    if reason == "slow::slow: :deadline-ms is declared but governs nothing in \
                        :satisfies mode (measured: neither the caller's wait nor this service's \
                        own outbound calls) — put the deadline at the CALL SITE with \
                        (:wat::service::call-by-deadline peer op <ms> <fallback>), or drop the \
                        clause."
            )
    );
}
