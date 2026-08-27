//! FM 2-bis probe — arc 258 Stone 258.1: `if` infers without a return annotation.
//!
//! `(:wat::core::if cond then else)` — 3 args — type-checks and evaluates with NO `-> :T`:
//! the condition must be `:bool`; the form's type is `unify(then, else)`; the consuming site
//! does the rest (recipient `assignable`, the `do` model). The mandatory mid-form `-> :T`
//! is dropped — the annotation was redundant (it `unify`d each branch against the declared
//! type, forcing the branches to unify with each other anyway).
//!
//! C01: bare `(if true 1 2)` evals to 1 — the annotation is gone and inference carries it.
//! C02: `(if false -> :i64 1 2)` evals to 2 — the annotated 5-arg form still works (dual-read).
//! C03: `(if true 1 "s")` is REJECTED for a branch-type mismatch (not arity) — inference checks.
//!
//! Run: `cargo test --release --test probe_arc258_stone1_if_inference`

use wat::freeze::startup_from_file;
use wat::runtime::{apply_function, Value};

fn eval_i64_file(path: &str) -> Result<i64, String> {
    let world = startup_from_file(path).map_err(|e| format!("startup/check: {e:?}"))?;
    let func = world.symbols().get(":user::compute").expect(":user::compute").clone();
    match apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
        .map_err(|e| format!("eval: {e:?}"))?
    {
        Value::i64(n) => Ok(n),
        other => Err(format!("non-i64: {other:?}")),
    }
}

#[test]
fn contract_01_bare_if_infers_and_evals() {
    assert_eq!(
        eval_i64_file("tests/types/probe_arc258_stone1_if_inference_c01.wat"),
        Ok(1),
        "bare `(if true 1 2)` infers via branch-unification and evals the then-branch"
    );
}

#[test]
fn contract_02_annotated_if_still_works() {
    assert_eq!(
        eval_i64_file("tests/types/probe_arc258_stone1_if_inference_c02.wat"),
        Ok(2),
        "the annotated 5-arg `(if cond -> :T then else)` keeps working (dual-read)"
    );
}

#[test]
fn contract_03_branch_mismatch_rejected_for_the_right_reason() {
    // then=:i64, else=:String — they do not unify. Inference must reject this as a branch
    // mismatch, NOT as an arity error ("now requires -> :T").
    let r = startup_from_file("tests/types/probe_arc258_stone1_if_inference_c03.wat.bad")
        .map(|_| ())
        .map_err(|e| format!("{e:?}"));
    // `.expect_err` folds the presence check into the discriminant check below — a bare
    // is_err() check here proved nothing about WHICH error (arc 296 Stone L); the golden-file
    // compare is the actual non-vacuous discriminant (full EDN structural match).
    let msg = r.expect_err("a branch-type mismatch must be rejected");
    wat::assert_edn_matches_file!(msg, "probe_arc258_stone1_if_inference__contract_03_branch_mismatch_rejected_for_the_right_reason.edn", "branch-type mismatch, not arity: TypeMismatch");
}
