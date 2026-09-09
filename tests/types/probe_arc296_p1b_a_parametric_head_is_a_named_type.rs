//! PROBE — P-1's wall never validates the HEAD of a parametric annotation.
//!
//! Found 2026-09-08 while building A-2's over-reach detector. **This is a hole in landed, pushed
//! work** (`f302681e7`), measured on the green tree at `b0a503f0c`:
//!
//! ```text
//!   [x <- (:usr::TotallyMadeUp :- [:wat::core::i64])]   check=0   ⛔ ACCEPTED
//!   [x <- :usr::TotallyMadeUp]                          check=1   refused
//!   [x <- (:wat::core::Option :- [:usr::TotallyMadeUp])] check=1   refused — ARGS are walked
//! ```
//!
//! ## The cause, and the lesson worth more than the fix
//!
//! `walk_type_expr` (`src/declare/typevar.rs`):
//!
//! ```text
//!   TypeExpr::Parametric { args, .. } => { for a in args { walk_type_expr(a, …) } }
//!                                  ^^ the HEAD is destructured away and never visited
//! ```
//!
//! ★ **That walk was built for free-VARIABLE collection**, where skipping the head is correct — a
//! head is not a type variable. P-1's wall reused it to validate NAMED TYPES, a different question,
//! and inherited a traversal shaped by the old one. My own BRIEF said *"share this recursion; do
//! not write a fifth walker"* — right in spirit, and sharing carried the blind spot across with the
//! shape. `[[feedback_a_pass_answers_only_the_question_the_instrument_asks]]`
//!
//! ⚠ The fix must NOT be a second walker. Give `walk_type_expr` a head visit that the free-var
//! caller ignores, exactly as it already gained `visit_var` in A-1 — one recursion, more callbacks.
//!
//! ## Why it matters beyond tidiness
//!
//! A variant annotation IS parametric — `(:usr::Box::Full :- [T])`. Until the head is validated,
//! A-2 cannot tell a real variant from a typo'd one, and its over-reach detector would be vacuous.
//!
//! The subject test is `#[ignore]`d and is UN-IGNORED BY THE STONE.

use std::path::PathBuf;
use std::process::{Command, Stdio};

fn check(case: &str) -> i32 {
    let p: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/types")
        .join(format!("probe_arc296_p1b_a_parametric_head_is_a_named_type__{case}.wat"));
    assert!(p.exists(), "fixture missing: {}", p.display());
    Command::new(env!("CARGO_BIN_EXE_wat"))
        .arg("--check").arg(&p)
        .stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped())
        .output().expect("spawn wat --check").status.code().unwrap_or(-1)
}

/// ⛔ THE WIDEST CONTROL. Every generic in the corpus is a parametric annotation — Option, Result,
/// Vector, HashMap. If this breaks, the stone broke the world rather than closing a hole.
#[test]
fn a_real_parametric_head_is_accepted() {
    assert_eq!(check("parametric_head_real"), 0);
}

/// CONTROL — P-1 already refuses the bare spelling of the very same name.
#[test]
fn the_same_phantom_bare_is_already_refused() {
    assert_eq!(check("bare_head_phantom"), 1);
}

/// CONTROL — a phantom in ARGUMENT position is already refused, which is what proves the walk
/// reaches args and misses only the head.
#[test]
fn a_phantom_in_argument_position_is_already_refused() {
    assert_eq!(check("parametric_arg_phantom"), 1);
}

/// SUBJECT — the head of a parametric annotation names nothing, and is accepted.
#[test]
fn a_phantom_parametric_head_is_refused() {
    assert_eq!(
        check("parametric_head_phantom"),
        1,
        "`:usr::TotallyMadeUp` is refused bare and must be refused as a parametric head; the \
         annotation position cannot be honest for one spelling and blind for the other"
    );
}
