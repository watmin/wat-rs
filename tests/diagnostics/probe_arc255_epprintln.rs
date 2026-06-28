//! Arc 255 — `:wat::kernel::epprintln` wiring + pretty-printing probe.
//!
//! Two pure (non-process) tests mirroring `probe_arc255_pprintln.rs`:
//!
//! 1. **pretty-output unit test** — `wat_edn::write_pretty` on a collection
//!    spans MORE THAN ONE line (same proof as pprintln; epprintln uses the
//!    same writer, just routed to stderr instead of stdout).
//!
//! 2. **type-check acceptance test** — freezes a minimal wat program that
//!    calls `(:wat::kernel::epprintln v)` and asserts the checker + startup
//!    succeed without error. Exercises all four wiring sites:
//!    `src/services/verbs.rs` (impl), `src/services/mod.rs` (re-export),
//!    `src/runtime.rs` (dispatch arm), and `src/check.rs` (∀T.T→nil scheme).

use wat::freeze::startup_beside;
use wat_edn::Keyword;

// ─── Test 1 — write_pretty produces multi-line output for a collection ────────
#[test]
fn epprintln_write_pretty_produces_multi_line_for_map() {
    use wat_edn::{Value, write, write_pretty};

    let map = Value::Map(vec![
        (Value::Keyword(Keyword::new("a")), Value::Integer(1)),
        (Value::Keyword(Keyword::new("b")), Value::Integer(2)),
    ]);

    let compact = write(&map);
    let pretty = write_pretty(&map);

    assert!(
        !compact.contains('\n'),
        "write (compact) must produce a single line for a 2-entry map; got: {:?}",
        compact
    );

    let line_count = pretty.lines().count();
    assert!(
        line_count > 1,
        "write_pretty must produce more than one line for a 2-entry map; \
         got {} line(s): {:?}",
        line_count,
        pretty
    );

    assert!(
        pretty.contains(":a") && pretty.contains(":b"),
        "write_pretty output must contain map keys :a and :b; got: {:?}",
        pretty
    );
}

// ─── Test 2 — type-checker accepts epprintln and startup succeeds ─────────────
// Wat source: co-located probe_arc255_epprintln.wat
#[test]
fn epprintln_type_checks_and_startup_succeeds() {
    match startup_beside(file!()) {
        Ok(_) => {}
        Err(e) => panic!(
            "(:wat::kernel::epprintln 42) must type-check and freeze without error — \
             check all four wiring sites (verbs.rs / mod.rs / runtime.rs / check.rs); \
             got: {}",
            e
        ),
    }
}
