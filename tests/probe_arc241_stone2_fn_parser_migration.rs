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

use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;

// ─── Helpers ──────────────────────────────────────────────────────────────────

/// Wrap the test fragment in a minimal main so `startup_from_source` is happy.
/// The fn-form under test lives in `src`; the main is just a nil placeholder.
fn with_nil_main(src: &str) -> String {
    format!(
        "{}\n(:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)",
        src
    )
}

/// Attempt to startup the source. Returns Ok(()) if the program parses,
/// type-checks, and freezes cleanly; Err(formatted error) otherwise.
fn try_startup(src: &str) -> Result<(), String> {
    let src = with_nil_main(src);
    startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .map(|_| ())
        .map_err(|e| format!("{:?}", e))
}

// ─── Contracts 1–4: A1/A2/A3 happy paths (well-formed fn signatures) ─────────

#[test]
fn contract_01_no_arg_fn_succeeds() {
    // (fn [] -> :T body) — empty argspec.
    let result = try_startup(
        r#"(:wat::core::define (:user::f -> :wat::core::i64)
             ((:wat::core::fn [] -> :wat::core::i64 42)))"#,
    );
    assert!(
        result.is_ok(),
        "well-formed no-arg fn should startup; got: {:?}",
        result
    );
}

#[test]
fn contract_02_single_arg_fn_succeeds() {
    // (fn [x <- :T] -> :T x)
    let result = try_startup(
        r#"(:wat::core::define (:user::f -> :wat::core::i64)
             ((:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 x) 7))"#,
    );
    assert!(
        result.is_ok(),
        "well-formed single-arg fn should startup; got: {:?}",
        result
    );
}

#[test]
fn contract_03_multi_arg_fn_succeeds() {
    // (fn [x <- :T y <- :T] -> :T body)
    let result = try_startup(
        r#"(:wat::core::define (:user::f -> :wat::core::i64)
             ((:wat::core::fn [x <- :wat::core::i64 y <- :wat::core::i64] -> :wat::core::i64
                (:wat::core::+ x y)) 3 4))"#,
    );
    assert!(
        result.is_ok(),
        "well-formed multi-arg fn should startup; got: {:?}",
        result
    );
}

#[test]
fn contract_04_let_bound_fn_succeeds() {
    // Let-binding a fn value — exercises A1 + A2/A3 paths together.
    let result = try_startup(
        r#"(:wat::core::define (:user::f -> :wat::core::i64)
             (:wat::core::let [g (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 x)]
               (g 42)))"#,
    );
    assert!(
        result.is_ok(),
        "let-bound fn should startup; got: {:?}",
        result
    );
}

// ─── Contracts 5–8: A1/A3 error paths (malformed signatures error cleanly) ───

#[test]
fn contract_05_name_not_symbol_errors() {
    // Slot 0 of triple is a keyword, not a Symbol. Canonical: NameNotSymbol.
    // Pre-migration: A1's inline walker emits "Expected Symbol at args_vec[0]" or similar.
    // Post-migration: canonical parser emits "name slot must be a plain symbol (not a
    // keyword, literal, or nested form)" via From<ArgSpecError> for RuntimeError.
    // Either way: ERROR (not silent success).
    let result = try_startup(
        r#"(:wat::core::define (:user::f -> :wat::core::i64)
             ((:wat::core::fn [:kw <- :wat::core::i64] -> :wat::core::i64 42)))"#,
    );
    assert!(
        result.is_err(),
        "non-Symbol at name slot must error; got Ok"
    );
}

#[test]
fn contract_06_missing_arrow_errors() {
    // Slot 1 of triple is `=` not `<-`. Canonical: MissingArrow.
    let result = try_startup(
        r#"(:wat::core::define (:user::f -> :wat::core::i64)
             ((:wat::core::fn [x = :wat::core::i64] -> :wat::core::i64 x)))"#,
    );
    assert!(
        result.is_err(),
        "missing `<-` arrow must error; got Ok"
    );
}

#[test]
fn contract_07_non_keyword_at_type_slot_errors() {
    // Slot 2 of triple is a string, not a Keyword. Canonical: TypeNotKeyword.
    let result = try_startup(
        r#"(:wat::core::define (:user::f -> :wat::core::i64)
             ((:wat::core::fn [x <- "i64"] -> :wat::core::i64 42)))"#,
    );
    assert!(
        result.is_err(),
        "non-Keyword at type slot must error; got Ok"
    );
}

#[test]
fn contract_08_incomplete_triple_errors() {
    // Argspec has fewer than 3 items at a triple position. Canonical: IncompleteTriple.
    // `[x <-]` — name then arrow but no type slot.
    let result = try_startup(
        r#"(:wat::core::define (:user::f -> :wat::core::i64)
             ((:wat::core::fn [x <-] -> :wat::core::i64 42)))"#,
    );
    assert!(
        result.is_err(),
        "incomplete triple must error; got Ok"
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
    let result = try_startup(
        r#"(:wat::core::define (:user::f -> :wat::core::i64)
             ((:wat::core::fn [x <- :wat::core::i64] :wat::core::i64 x)))"#,
    );
    assert!(
        result.is_err(),
        "missing `->` ret-arrow must error; got Ok"
    );
}

#[test]
fn contract_10_non_keyword_ret_type_errors() {
    // Argspec is fine; `->` is present; ret-type slot is a string not a Keyword.
    // Inline ret-clause check at A1/A2/A3 (unchanged by Stone 241.2).
    let result = try_startup(
        r#"(:wat::core::define (:user::f -> :wat::core::i64)
             ((:wat::core::fn [x <- :wat::core::i64] -> "i64" x)))"#,
    );
    assert!(
        result.is_err(),
        "non-Keyword ret-type must error; got Ok"
    );
}
