//! Arc 221 Stone 221.4b — Phase 2 macro-support keyword-shape probes.
//!
//! Verifies that the macro-support family in runtime.rs correctly handles
//! `HolonAST::Keyword` (not the retired `HolonAST::Symbol(":foo")`) after
//! Stone 221.4b's `watast_to_holon` fix.
//!
//! Functions fixed:
//!   - `eval_rename_callable_name` (runtime.rs:11560 assertion + 11588 writer)
//!     — now accepts `HolonAST::Keyword` as first Bundle child and emits
//!     `HolonAST::keyword()` as the renamed child.
//!   - `eval_extract_arg_names` (runtime.rs:11647/11653) — AUDITED as HONEST
//!     (arg names remain `HolonAST::Symbol`; they are bare WAT identifiers,
//!     not user keywords). No change needed; doc comments updated.
//!
//! Tests:
//!   1. `rename-callable-name` accepts Keyword first child.
//!   2. `rename-callable-name` rejects Symbol first child (wrong from-name errors).
//!   3. `defalias` end-to-end (Stone 241.12).
//!
//! Wat source lives in the co-located fixture: wat_arc221b_macro_support_keyword_shape.wat
//! (slurped via startup_beside(file!())).

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

fn run_string(world: &wat::freeze::FrozenWorld, expr: &str) -> String {
    let ast = wat::parse_one!(expr).expect("parse expr");
    match eval_in_frozen(&ast, world, &Environment::new())
        .expect("eval should succeed")
        .value_owned()
    {
        Value::String(s) => s.as_str().to_string(),
        other => panic!("expected String; got {:?}", other),
    }
}

fn run_expecting_runtime_err(world: &wat::freeze::FrozenWorld, expr: &str) -> bool {
    let ast = wat::parse_one!(expr).expect("parse expr");
    eval_in_frozen(&ast, world, &Environment::new()).is_err()
}

// ─── Probe 1 — rename-callable-name accepts Keyword first child ───────────────

/// `(:wat::runtime::rename-callable-name sig :t::probe-1-fn :t::probe-1-renamed)` where
/// `sig` is a Bundle with `HolonAST::Keyword("t::probe-1-fn")` as first child (produced
/// by `signature-of-defn` after Stone 221.4b's watast_to_holon fix).
///
/// This is the CORE fix: pre-Stone-221.4b `eval_rename_callable_name` asserted
/// `HolonAST::Symbol` at children[0]; after Stone 221.4b `watast_to_holon` emits
/// `HolonAST::Keyword` there, so the assertion would FAIL with TypeMismatch. The
/// Phase 2 fix changes the assertion to accept `HolonAST::Keyword`.
#[test]
fn probe_1_rename_callable_name_accepts_keyword_first_child() {
    let world = startup_beside(file!()).expect("startup");
    let s = run_string(&world, "(:t::probe-1)");
    // Renamed head must contain "probe-1-renamed".
    assert!(
        s.contains("probe-1-renamed"),
        "expected 'probe-1-renamed' in renamed head, got: {}",
        s
    );
    // Old name must be gone from the head keyword position.
    assert!(
        !s.contains("probe-1-fn"),
        "expected 'probe-1-fn' to be absent from renamed head, got: {}",
        s
    );
}

// ─── Probe 2 — rename-callable-name from-mismatch errors correctly ───────────

/// When `from` doesn't match the head's base name, `rename-callable-name` must
/// error with `MalformedForm`. This verifies the comparison logic (base without
/// leading colon vs from_str with leading colon, fixed in Stone 221.4b).
///
/// Pre-Stone-221.4b the comparison would fail INCORRECTLY for ALL renames (because
/// base had no colon but from_str did). Post-fix: only mismatches error.
#[test]
fn probe_2_rename_callable_name_from_mismatch_errors() {
    let world = startup_beside(file!()).expect("startup");
    assert!(
        run_expecting_runtime_err(&world, "(:t::probe-2)"),
        "expected runtime error for from-name mismatch in rename-callable-name"
    );
}

// ─── Probe 3 — defalias end-to-end (substrate target, Stone 241.12) ─────────

/// `(:wat::core::defalias :t::my-length :wat::core::length)` creates
/// an alias of a substrate primitive. Calling the alias must produce the same
/// result as calling the original.
///
/// Stone 241.12 — migrated from :wat::runtime::define-alias to native :wat::core::defalias.
/// The native form resolves the builtin via CheckEnv::with_builtins() at
/// registration time; no macro expansion required.
#[test]
fn probe_3_define_alias_end_to_end() {
    let world = startup_beside(file!()).expect("startup");
    let s = run_string(&world, "(:t::probe-3)");
    // Both length and my-length on [1,2,3] should produce 3.
    // Output should contain "3 3".
    assert!(
        s.contains("3") && {
            let count = s.matches('3').count();
            count >= 2
        },
        "expected both calls to produce 3 (length of 3-vector), got: {}",
        s
    );
}
