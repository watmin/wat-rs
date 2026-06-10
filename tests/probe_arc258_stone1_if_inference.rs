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

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

fn eval_i64(body: &str) -> Result<i64, String> {
    let src = format!(
        "(:wat::core::defn :user::compute [] -> :wat::core::i64 {body})\n\
         (:wat::core::defn :user::main [] -> :wat::core::nil nil)",
    );
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .map_err(|e| format!("startup/check: {e:?}"))?;
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    match eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .map_err(|e| format!("eval: {e:?}"))?
    {
        Value::i64(n) => Ok(n),
        other => Err(format!("non-i64: {other:?}")),
    }
}

/// Returns Ok(()) if the source type-checks, else Err(<diagnostic string>).
fn check_src(body: &str) -> Result<(), String> {
    let src = format!(
        "(:wat::core::defn :user::compute [] -> :wat::core::i64 {body})\n\
         (:wat::core::defn :user::main [] -> :wat::core::nil nil)",
    );
    startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .map(|_| ())
        .map_err(|e| format!("{e:?}"))
}

#[test]
fn contract_01_bare_if_infers_and_evals() {
    assert_eq!(
        eval_i64("(:wat::core::if true 1 2)"),
        Ok(1),
        "bare `(if true 1 2)` infers via branch-unification and evals the then-branch"
    );
}

#[test]
fn contract_02_annotated_if_still_works() {
    assert_eq!(
        eval_i64("(:wat::core::if false -> :wat::core::i64 1 2)"),
        Ok(2),
        "the annotated 5-arg `(if cond -> :T then else)` keeps working (dual-read)"
    );
}

#[test]
fn contract_03_branch_mismatch_rejected_for_the_right_reason() {
    // then=:i64, else=:String — they do not unify. Inference must reject this as a branch
    // mismatch, NOT as an arity error ("now requires -> :T").
    let r = check_src(r#"(:wat::core::if true 1 "s")"#);
    assert!(r.is_err(), "a branch-type mismatch must be rejected");
    let msg = r.unwrap_err();
    assert!(
        !msg.contains("now requires"),
        "must reject for branch-type mismatch, not arity; got: {msg}"
    );
}
