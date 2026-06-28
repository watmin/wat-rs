//! FM 2-bis probe — arc 258 Stone 258.2a: `cond` reborn as a wat macro over bare `if`.
//!
//! cond's honest form: a `defmacro` in core.wat expanding `(cond (test body)… (:else body))`
//! to nested bare `if` — no `-> :T` annotation, the Rust special form shadowed (and, in 258.2b,
//! annihilated). Bare cond (no annotation) is rejected at HEAD because the Rust `infer_cond`
//! demands `-> :T` at position 1; after the macro, it expands to `if` and evaluates.
//!
//! C01: `(cond ((= 1 1) 10) (:else 20))` evals to 10 — first arm taken, no annotation.
//! C02: `(cond ((= 1 2) 10) (:else 20))` evals to 20 — falls through to :else.
//! C03: 3-arm — `(cond ((= 1 2) 10) ((= 2 2) 20) (:else 30))` evals to 20 — proves fixpoint recursion.
//!
//! Wat source lives in the co-located fixture: probe_arc258_stone2_cond_macro.wat
//! (slurped via startup_beside(file!())).
//!
//! Run: `cargo test --release --test probe_arc258_stone2_cond_macro`

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

fn eval_named_i64(world: &wat::freeze::FrozenWorld, fn_call: &str) -> Result<i64, String> {
    let ast = wat::parse_one!(fn_call).expect("parse");
    match eval_in_frozen(&ast, world, &Environment::new())
        .map(|tv| tv.value_owned())
        .map_err(|e| format!("eval: {e:?}"))?
    {
        Value::i64(n) => Ok(n),
        other => Err(format!("non-i64: {other:?}")),
    }
}

#[test]
fn contract_01_first_arm_taken() {
    let world = startup_beside(file!()).expect("startup");
    assert_eq!(
        eval_named_i64(&world, "(:user::compute-1)"),
        Ok(10),
        "bare cond (no -> :T) expands to nested if; the first true arm is taken"
    );
}

#[test]
fn contract_02_else_fallthrough() {
    let world = startup_beside(file!()).expect("startup");
    assert_eq!(
        eval_named_i64(&world, "(:user::compute-2)"),
        Ok(20),
        "no arm matches → the :else body"
    );
}

#[test]
fn contract_03_three_arm_recursion() {
    let world = startup_beside(file!()).expect("startup");
    assert_eq!(
        eval_named_i64(&world, "(:user::compute-3)"),
        Ok(20),
        "a middle arm is taken — proves the macro re-expands to fixpoint across N arms"
    );
}
