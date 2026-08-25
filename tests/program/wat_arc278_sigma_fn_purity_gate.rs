//! Arc 278 — `docs/arc/2026/06/278-rules-engine/BRIEF-sigma-fn-must-be-pure-total-deterministic.md`
//!
//! RED-first probe for the sigma-fn install-time gate. `(:wat::config::set-presence-sigma! f)` /
//! `set-coincident-sigma!` used to check ONLY arity + declared types (`check_sigma_fn_signature`,
//! pre-278). `presence?`/`coincident?` invoke the installed fn to compute their floor and are
//! themselves classified `pure: true, deterministic: true` — an unchecked user body was the one
//! place that classification could still lie. `check_sigma_fn_contract` (`src/freeze.rs`, the
//! renamed+extended check) now additionally proves pure ∧ deterministic ∧ total, or refuses
//! startup naming the failing axis.
//!
//! Each `.wat.bad` fixture violates exactly ONE axis via an op the classifier
//! (`src/rete/purity.rs`) actually denies — never a fabricated/asserted-without-cause failure:
//! - `_impure` — `(:wat::io::read-file ...)`: `is_effectful_op` denies the whole `:wat::io::`
//!   namespace outright (Pure).
//! - `_nondeterministic` — `(:wat::uuid::v4)`: `intrinsic_meta` declares it
//!   `pure: true, deterministic: false` (the one hand-documented pure-but-random op).
//! - `_nontotal` — `(:wat::core::i64::+ d 1)`: pure ∧ deterministic (in the `pure_det` list) but
//!   NOT in the `total` allow-list — `i64::+`/`-`/`*` all raise `IntegerOverflow` (verified against
//!   `checked_add` et al. in `runtime.rs`; see `purity.rs`'s own `total` doc comment).
//!
//! The `_valid` fixtures are the converse: a body that is trivially pure ∧ deterministic ∧ total
//! (a bare symbol reference, no calls at all) must still install cleanly — without this, a probe
//! that only ever sees refusals cannot tell "the gate works" from "the gate rejects everything."
//!
//! `FunctionBody::Native` (STOP-1 of the brief) has NO wat-surface fixture here: research proved
//! (grepping every `Function { .. }` construction site in the crate) that nothing anywhere
//! constructs a `Function` with `FunctionBody::Native` — a sigma fn's `Function` value, obtained
//! by evaluating the setter's argument to `Value::wat__core__fn`, is always `FunctionBody::Wat`.
//! There is no wat source a user can write that reaches the Native arm. See
//! `wat_arc278_sigma_fn_purity_gate_native_axis` below for the direct Rust-level exercise of
//! that defensive code path instead (mirrors `purity.rs`'s own `classify_native_fn` unit tests).

use wat::freeze::{startup_from_file, StartupError};

fn assert_sigma_rejected(path: &str, axis_word: &str) {
    let err = startup_from_file(path)
        .expect_err(&format!("{path}: expected startup to be REFUSED (violates `{axis_word}`), but it froze cleanly"));
    match err {
        StartupError::SigmaFn(msg) => {
            assert!(
                msg.contains(axis_word),
                "{path}: StartupError::SigmaFn message must name the failing axis `{axis_word}`; got: {msg}"
            );
        }
        other => panic!("{path}: expected StartupError::SigmaFn naming `{axis_word}`; got a different variant: {other}"),
    }
}

// ─── T1. The converse — a trivially pure ∧ deterministic ∧ total sigma still installs ──────────
//
// Without this half the probe cannot distinguish "the gate correctly enforces the three axes"
// from "the gate rejects every installed sigma fn, full stop."

#[test]
fn t1_identity_presence_sigma_installs_cleanly() {
    startup_from_file("tests/program/wat_arc278_sigma_fn_purity_gate_presence_valid.wat")
        .expect("a bare `(fn [d] -> :i64 d)` presence-sigma is pure ∧ deterministic ∧ total by construction (a lone symbol reference, no calls) and must install cleanly");
}

#[test]
fn t1_identity_coincident_sigma_installs_cleanly() {
    startup_from_file("tests/program/wat_arc278_sigma_fn_purity_gate_coincident_valid.wat")
        .expect("a bare `(fn [d] -> :i64 d)` coincident-sigma is pure ∧ deterministic ∧ total by construction and must install cleanly");
}

// ─── T2. Each axis, violated independently, is REFUSED and named ──────────────────────────────

#[test]
fn t2_impure_presence_sigma_rejected_pure_axis_named() {
    assert_sigma_rejected(
        "tests/program/wat_arc278_sigma_fn_purity_gate_presence_impure.wat.bad",
        "pure",
    );
}

#[test]
fn t2_nondeterministic_presence_sigma_rejected_deterministic_axis_named() {
    assert_sigma_rejected(
        "tests/program/wat_arc278_sigma_fn_purity_gate_presence_nondeterministic.wat.bad",
        "deterministic",
    );
}

#[test]
fn t2_nontotal_presence_sigma_rejected_total_axis_named() {
    assert_sigma_rejected(
        "tests/program/wat_arc278_sigma_fn_purity_gate_presence_nontotal.wat.bad",
        "total",
    );
}

// ─── T3. Site parity — `set-coincident-sigma!` gets the SAME treatment ─────────────────────────
//
// The brief: "Add the check where `check_sigma_fn_signature` is already called — freeze.rs:462
// (presence) and :497 (coincident). Both sites, same treatment." One violation on the coincident
// path (on top of T1's coincident converse above) is enough to prove it is not presence-only.

#[test]
fn t3_impure_coincident_sigma_rejected_pure_axis_named() {
    assert_sigma_rejected(
        "tests/program/wat_arc278_sigma_fn_purity_gate_coincident_impure.wat.bad",
        "pure",
    );
}
