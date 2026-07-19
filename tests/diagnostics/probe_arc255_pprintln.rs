//! Arc 255 — `:wat::kernel::pprintln` wiring + pretty-printing probe.
//!
//! Two pure (non-process) tests:
//!
//! 1. **pretty-output unit test** — directly exercises `wat_edn::write_pretty`
//!    on a collection value and asserts the output spans MORE THAN ONE line.
//!    This proves the core behavior difference between `pprintln` (pretty,
//!    multi-line) and `println` (compact, single line) without requiring a
//!    subprocess. A `Value::Map` with entries always breaks multi-line in
//!    `write_pretty`; a `Value::Vector` of all-scalar elements ≤ 8 stays
//!    inline (same as compact) — the map covers the interesting case.
//!
//! 2. **type-check acceptance test** — freezes a minimal wat program that
//!    calls `(:wat::kernel::pprintln v)` and asserts the checker + startup
//!    succeed without error. This exercises all four wiring sites:
//!    `src/services/verbs.rs` (impl), `src/services/mod.rs` (re-export),
//!    `src/runtime.rs` (dispatch arm), and `src/check.rs` (∀T.T→nil scheme).
//!
//! Ambient stdout capture across a subprocess boundary (the `run-hermetic`
//! path) is excluded from the nursery (which is pure/non-process by contract).
//! The process-level stdout capture proof lives in the integration test at
//! `tests/probe_run_hermetic_ast_stdout_capture.rs` (the established pattern
//! for that surface).

use wat::freeze::startup_beside;
use wat_edn::Keyword;

// ─── Test 1 — write_pretty produces multi-line output for a collection ────────
//
// `(:wat::kernel::pprintln v)` calls `wat_edn::write_pretty(&edn)`.
// Prove that `write_pretty` of a non-trivial collection value spans
// multiple lines (proving pprintln ≠ println for such values).
//
// A `Value::Map` with two or more entries always breaks to multi-line in
// `write_pretty` (each key-value pair on its own indented line). Both
// `write` (compact) and `write_pretty` (pretty) produce a single string;
// the difference is whether that string contains embedded newlines.
#[test]
fn pprintln_write_pretty_produces_multi_line_for_map() {
    use wat_edn::{Value, write, write_pretty};

    // A two-entry map — any non-empty map breaks multi-line in write_pretty.
    let map = Value::Map(vec![
        (Value::Keyword(Keyword::new("a")), Value::Integer(1)),
        (Value::Keyword(Keyword::new("b")), Value::Integer(2)),
    ]);

    let compact = write(&map);
    let pretty = write_pretty(&map);

    // Compact: one line, no embedded newlines.
    assert!(
        !compact.contains('\n'),
        "write (compact) must produce a single line for a 2-entry map; got: {:?}",
        compact
    );

    // Pretty: multiple lines (each k-v pair on its own line + opening/closing braces).
    let line_count = pretty.lines().count();
    assert!(
        line_count > 1,
        "write_pretty must produce more than one line for a 2-entry map; \
         got {} line(s): {:?}",
        line_count,
        pretty
    );

    // Pretty output must be the exact multi-line form.
    assert_eq!(
        pretty,
        // rune:lint(no-inlined-edn) — is the EDN tooling correct: exact-format pretty-printer output under test; a structural comparison would defeat the whitespace assertion
        "{\n  :a 1\n  :b 2\n}",
        "write_pretty output must be the exact multi-line form"
    );
}

// ─── Test 2 — type-checker accepts pprintln and startup succeeds ──────────────
//
// Freeze a minimal wat program that calls `(:wat::kernel::pprintln v)`.
// If any of the four wiring sites (verbs.rs / mod.rs / runtime.rs / check.rs)
// is missing, the checker rejects the call with an unresolved-verb error and
// startup returns Err. A clean Ok proves the full wiring.
//
// The program calls pprintln on a :wat::core::i64 literal (the ∀T scheme
// means any T is accepted). The body returns nil so :user::main type-checks.
// Wat source: co-located probe_arc255_pprintln.wat
#[test]
fn pprintln_type_checks_and_startup_succeeds() {
    match startup_beside(file!()) {
        Ok(_) => {}
        Err(e) => panic!(
            "(:wat::kernel::pprintln 42) must type-check and freeze without error — \
             check all four wiring sites (verbs.rs / mod.rs / runtime.rs / check.rs); \
             got: {}",
            e
        ),
    }
}
