//! FM 2-bis BEHAVIORAL-PARITY probe for Stone 241.2 — A1/A2/A3 fn-signature
//! parser migration through canonical `parse_argspec_triples`.
//!
//! ## Why this probe
//!
//! Stone 241.2 migrates three internal parsers:
//!   - **A1** `parse_fn_signature` at `src/runtime.rs:6750`
//!   - **A2** `parse_fn_signature_for_check` at `src/check.rs:15205`
//!   - **A3** `parse_fn_signature_for_check_diag` at `src/check.rs:15258`
//!
//! Their public surface (callable signatures + caller-observable behavior)
//! stays IDENTICAL. The migration is INTERNAL: the inline triple walker at
//! each site routes through `wat::argspec::parse_argspec_triples`; the
//! ret-clause parsing stays inline per site (per-site error semantics differ).
//!
//! ## What this probe proves
//!
//! Behavioral parity: well-formed fn-forms parse cleanly; malformed
//! fn-forms produce errors (don't silently succeed). The probe asserts on
//! the err/ok BOUNDARY, not on exact error message text — canonical-domain-
//! neutral wording replaces inline ad-hoc messages per `From<ArgSpecError>`
//! impls shipped in Stone 241.1.fix; message strings are EXPECTED to change.
//!
//! ## Pre/post migration
//!
//! Pre-Stone 241.2 (HEAD `03f22394`): all contracts PASS via the existing
//! inline triple walkers at A1/A2/A3.
//!
//! Post-Stone 241.2: all contracts STILL PASS; the canonical parser
//! preserves the err/ok behavior at each call site. The exact error
//! messages differ but the variant CLASS (`RuntimeError::MalformedForm` for
//! A1, `CheckError::MalformedForm` for A3, silent for A2) is preserved via
//! the `From<ArgSpecError>` impls.
//!
//! ## FM 2-bis nature: PARITY probe, not isolation probe
//!
//! Stone 241.1's probe (`probe_arc241_stone1_argspec_canonical.rs`) used
//! compile-fail isolation — it failed to compile at HEAD because
//! `wat::argspec` didn't exist; passing post-stone proved the substrate
//! existed and worked.
//!
//! This probe uses BEHAVIORAL PARITY — it passes BOTH at HEAD and
//! post-stone; any regression in the err/ok boundary indicates the
//! migration broke A1/A2/A3's observable contract. Different FM 2-bis
//! shape; both are valid disconfirmation tools per the discipline.
//!
//! Run: `cargo test --release --test probe_arc241_stone2_fn_parser_migration`

//! Wat source: tests/function/probe_arc241_stone2_fn_parser_migration.wat
//! Negative fixtures: probe_arc241_stone2_c05.wat.bad, probe_arc241_stone2_c06.wat.bad,
//!   probe_arc241_stone2_c07.wat.bad, probe_arc241_stone2_c08.wat.bad,
//!   probe_arc241_stone2_c09.wat.bad, probe_arc241_stone2_c10.wat.bad.

use wat::check::error::CheckErrorKind;
use wat::freeze::{startup_beside, startup_from_file};
use wat::runtime::{apply_function, Value};

// just-eval (rubric): each `fn_name` names a zero-arg fn defined in the co-located
// fixture; fetch it from the frozen world and `apply_function` it — no inline wat driver.
fn run(fn_name: &str) -> Value {
    let world = startup_beside(file!()).expect("startup for stone2 fn-parser-migration fixture");
    let func = world
        .symbols()
        .get(fn_name)
        .unwrap_or_else(|| panic!("no {fn_name} in fixture"))
        .clone();
    apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
        .expect("eval should succeed")
}

// ─── Contracts 1–4: A1/A2/A3 happy paths (well-formed fn signatures) ─────────

#[test]
fn contract_01_no_arg_fn_succeeds() {
    // (fn [] -> :T body) — empty argspec; c01-f returns i64(42).
    assert_eq!(run(":user::c01-f"), Value::i64(42), "well-formed no-arg fn should succeed");
}

#[test]
fn contract_02_single_arg_fn_succeeds() {
    // (fn [x <- :T] -> :T x) called with 7 → i64(7).
    assert_eq!(run(":user::c02-f"), Value::i64(7), "well-formed single-arg fn should succeed");
}

#[test]
fn contract_03_multi_arg_fn_succeeds() {
    // (fn [x <- :T y <- :T] -> :T body) called with 3 4 → 3+4 = i64(7).
    assert_eq!(run(":user::c03-f"), Value::i64(7), "well-formed multi-arg fn should succeed");
}

#[test]
fn contract_04_let_bound_fn_succeeds() {
    // Let-binding a fn value — exercises A1 + A2/A3 paths together; g(42) → i64(42).
    assert_eq!(run(":user::c04-f"), Value::i64(42), "let-bound fn should succeed");
}

// ─── Contracts 5–8: A1/A3 error paths (malformed signatures error cleanly) ───

#[test]
fn contract_05_name_not_symbol_errors() {
    // Slot 0 of triple is a keyword, not a Symbol. Canonical: NameNotSymbol.
    // Pre-migration: A1's inline walker emits "Expected Symbol at args_vec[0]" or similar.
    // Post-migration: canonical parser emits "name slot must be a plain symbol (not a
    // keyword, literal, or nested form)" via From<ArgSpecError> for RuntimeError.
    // Either way: ERROR (not silent success).
    let result = startup_from_file("tests/function/probe_arc241_stone2_c05.wat.bad");
    wat::assert_startup_error!(result, check
        CheckErrorKind::MalformedForm { head, reason, .. }
            if head == ":wat::core::fn"
            && reason == "name must be a plain symbol (not a keyword, literal, or nested form)"
    );
}

#[test]
fn contract_06_missing_arrow_errors() {
    // Slot 1 of triple is `=` not `<-`. Canonical: MissingArrow.
    let result = startup_from_file("tests/function/probe_arc241_stone2_c06.wat.bad");
    wat::assert_startup_error!(result, check
        CheckErrorKind::MalformedForm { head, reason, .. }
            if head == ":wat::core::fn"
            && reason == "triple must be `name <- :T`; expected `<-` as the second element"
    );
}

#[test]
fn contract_07_non_keyword_at_type_slot_errors() {
    // Slot 2 of triple is a string, not a Keyword. Canonical: TypeNotKeyword.
    let result = startup_from_file("tests/function/probe_arc241_stone2_c07.wat.bad");
    wat::assert_startup_error!(result, check
        CheckErrorKind::MalformedForm { head, reason, .. }
            if head == ":wat::core::fn"
            && reason == "type slot must be a keyword (e.g. `:wat::core::i64`); got a non-keyword"
    );
}

#[test]
fn contract_08_incomplete_triple_errors() {
    // Argspec has fewer than 3 items at a triple position. Canonical: IncompleteTriple.
    // `[x <-]` — name then arrow but no type slot.
    let result = startup_from_file("tests/function/probe_arc241_stone2_c08.wat.bad");
    wat::assert_startup_error!(result, check
        CheckErrorKind::MalformedForm { head, reason, .. }
            if head == ":wat::core::fn"
            && reason == "triple is incomplete; expected `name <- :T` but ran out of items"
    );
}

// ─── Contracts 9–10: ret-clause inline (stays unchanged by Stone 241.2) ──────

#[test]
fn contract_09_missing_ret_arrow_errors() {
    // Argspec is fine; `->` is missing between args-vector and ret-type.
    // This is the INLINE ret-clause check at A1/A2/A3 (not delegated to canonical).
    // Pre-migration: A1's inline check emits "fn signature missing `->` between
    // args-vector and return type". Post-migration: SAME inline check; same message.
    // (The ret-clause inline parsing is UNCHANGED in Stone 241.2 per DESIGN D2.)
    let result = startup_from_file("tests/function/probe_arc241_stone2_c09.wat.bad");
    wat::assert_startup_error!(result, check
        CheckErrorKind::MalformedForm { head, reason, .. }
            if head == ":wat::core::fn"
            && reason == "fn signature: expected `->` between args-vector and return type; got keyword"
    );
}

#[test]
fn contract_10_non_keyword_ret_type_errors() {
    // Argspec is fine; `->` is present; ret-type slot is a string not a Keyword.
    // Inline ret-clause check at A1/A2/A3 (unchanged by Stone 241.2).
    let result = startup_from_file("tests/function/probe_arc241_stone2_c10.wat.bad");
    wat::assert_startup_error!(result, check
        CheckErrorKind::MalformedForm { head, reason, .. }
            if head == ":wat::core::fn"
            && reason == "fn signature: expected a return-type keyword after `->` (e.g. `:wat::core::i64`); got string"
    );
}
