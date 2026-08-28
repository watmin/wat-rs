//! FM 2-bis BEHAVIORAL-PARITY probe for Stone 241.3 — A4 defclause parser
//! migration through canonical `parse_argspec_triples`.
//!
//! ## Why this probe
//!
//! Stone 241.3 migrates the single internal parser:
//!   - **A4** `parse_defclause_args` at `src/runtime.rs:6827`
//!
//! A4's public signature stays IDENTICAL (`(args_vec, head, form_span) ->
//! Result<Vec<(String, TypeExpr)>, RuntimeError>`). The migration is INTERNAL:
//! the 69-line inline triple walker is replaced with a 7-line canonical call;
//! `?` converts `ArgSpecError` → `RuntimeError` via the `From<>` impl shipped
//! in Stone 241.1.fix; `spec.fixed_params` is returned DIRECTLY (no unzip
//! needed — defclause's return shape IS the canonical's fixed_params shape).
//!
//! ## What this probe proves
//!
//! Behavioral parity: well-formed defclause forms parse cleanly; malformed
//! defclause forms produce errors (don't silently succeed). The probe asserts
//! on the err/ok BOUNDARY, not on exact error message text — canonical-
//! domain-neutral wording replaces inline arc-lineage citations (e.g. "literal
//! patterns are not permitted (arc 159/169/234 binding contract requires a
//! plain symbol name)" → "name slot must be a plain symbol (not a keyword,
//! literal, or nested form)"); the message changes but the err/ok boundary
//! stays.
//!
//! ## Pre/post migration
//!
//! Pre-Stone 241.3 (HEAD `21877135`): all contracts PASS via the existing
//! inline triple walker at A4.
//!
//! Post-Stone 241.3: all contracts STILL PASS; the canonical parser
//! preserves the err/ok behavior. The exact error messages differ but the
//! variant CLASS (`RuntimeError::MalformedForm`) is preserved via
//! `From<ArgSpecError> for RuntimeError`.
//!
//! ## FM 2-bis nature: PARITY probe (same shape as Stone 241.2)
//!
//! Mirrors Stone 241.2's behavioral-parity discipline. Passes BOTH at HEAD
//! and post-stone; regression in err/ok boundary indicates migration broke.
//!
//! ## Phase 1 closure note
//!
//! Stone 241.3 closes the parser-divergence class. After this stone, all 4
//! triple walkers (A1/A2/A3/A4) route through ONE canonical parser. The
//! substrate carries ONE triple-walking implementation; same structural
//! failures produce same `ArgSpecError` variants; per-site error conversion
//! at the call boundary via `From<>` impls.
//!
//! Run: `cargo test --release --test probe_arc241_stone3_defclause_parser_migration`

//! Wat source: tests/function/probe_arc241_stone3_defclause_parser_migration.wat
//! Negative fixtures: probe_arc241_stone3_c04.wat.bad, probe_arc241_stone3_c05.wat.bad,
//!   probe_arc241_stone3_c06.wat.bad.

use wat::freeze::{startup_beside, startup_from_file, StartupError};
use wat::runtime::{apply_function, RuntimeErrorKind, Value};

/// just-eval (rubric): fetch `fn_name` from the co-located fixture (`startup_beside`)
/// and `apply_function` it with `args` — no inline wat driver.
fn run(fn_name: &str, args: Vec<Value>) -> Value {
    let world = startup_beside(file!()).expect("startup for stone3 defclause-parser-migration fixture");
    let func = world
        .symbols()
        .get(fn_name)
        .unwrap_or_else(|| panic!("no {fn_name} in fixture"))
        .clone();
    apply_function(func, args, world.symbols(), wat::rust_caller_span!())
        .expect("eval should succeed")
}

// ─── Contracts 1–3: A4 happy paths (well-formed defclause args) ──────────────

#[test]
fn contract_01_defclause_no_args_succeeds() {
    // (defclause [] -> :T body) — empty argspec; c01-f() → i64(42).
    let result = run(":user::c01-f", vec![]);
    assert_eq!(result, Value::i64(42), "well-formed no-arg defclause should return i64(42)");
}

#[test]
fn contract_02_defclause_single_arg_succeeds() {
    // c02-f is a 1-arg defclause: call with i64(7) → i64(7).
    let result = run(":user::c02-f", vec![Value::i64(7)]);
    assert_eq!(result, Value::i64(7), "well-formed single-arg defclause should return i64(7)");
}

#[test]
fn contract_03_defclause_multi_arg_succeeds() {
    // c03-f is a 2-arg defclause: call with 3 4 → 3+4 = i64(7).
    let result = run(":user::c03-f", vec![Value::i64(3), Value::i64(4)]);
    assert_eq!(result, Value::i64(7), "well-formed multi-arg defclause should return i64(7)");
}

// ─── Contracts 4–6: A4 error paths (malformed argspecs error cleanly) ───────

#[test]
fn contract_04_name_not_symbol_errors() {
    // Slot 0 of triple is a keyword, not a Symbol.
    // A4 enforces this per arc 159/169/234 binding contract; canonical also enforces.
    let result = startup_from_file("tests/function/probe_arc241_stone3_c04.wat.bad");
    wat::assert_startup_error!(result,
        StartupError::Runtime(e) if matches!(
            e.kind(),
            RuntimeErrorKind::MalformedForm { head, reason }
                if head == ":wat::core::defclause"
                && reason == "name must be a plain symbol (not a keyword, literal, or nested form)"
        )
    );
}

#[test]
fn contract_05_missing_arrow_errors() {
    // Slot 1 of triple is `=` not `<-`.
    let result = startup_from_file("tests/function/probe_arc241_stone3_c05.wat.bad");
    wat::assert_startup_error!(result,
        StartupError::Runtime(e) if matches!(
            e.kind(),
            RuntimeErrorKind::MalformedForm { head, reason }
                if head == ":wat::core::defclause"
                && reason == "triple must be `name <- :T`; expected `<-` as the second element"
        )
    );
}

#[test]
fn contract_06_incomplete_triple_errors() {
    // Argspec has fewer than 3 items at a triple position.
    let result = startup_from_file("tests/function/probe_arc241_stone3_c06.wat.bad");
    wat::assert_startup_error!(result,
        StartupError::Runtime(e) if matches!(
            e.kind(),
            RuntimeErrorKind::MalformedForm { head, reason }
                if head == ":wat::core::defclause"
                && reason == "triple is incomplete; expected `name <- :T` but ran out of items"
        )
    );
}
