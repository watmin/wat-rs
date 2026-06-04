//! FM-2-bis probe for Arc 249 — threading macros `->` (thread-first) + `->>` (thread-last).
//!
//! The missing capability: Clojure's threading forms. `(->> x s1 s2)` desugars to
//! `(s2 (s1 x))` — a left fold of the accumulator through each step, injecting it
//! as the LAST arg (`->>`) or FIRST arg (`->`) of each step form. This is a pure
//! macro-expansion-time source-to-source rewrite (src/macros.rs `expand_form`,
//! mirroring the `:wat::core::keyword/of` built-in at macros.rs:548-569); it
//! desugars to ordinary nested calls BEFORE type-check, so the checker/runtime
//! never see `->`/`->>` and need no changes.
//!
//! ROW STATUS:
//!   - REGRESSION (GREEN at HEAD + after): plain fn-first `(map f xs)` — no threading.
//!     Anchors the harness + the arc-247 fn-first map the threading sits on top of.
//!   - MINT (RED at HEAD; `->`/`->>` are unrecognized bare-symbol heads → check/eval
//!     error; `#[ignore]`'d): un-ignored by sonnet after the desugar lands.
//!
//! Disconfirmation at HEAD: run the mints explicitly —
//!   cargo test --release --test probe_arc249_threading -- --ignored
//! every mint must FAIL (threading does not exist yet). Done when all pass with
//! zero `#[ignore]` and zero `-- --ignored` needed.
//!
//! Run: cargo test --release --test probe_arc249_threading

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

fn with_nil_main(src: &str) -> String {
    format!(
        "{}\n(:wat::core::defn :user::main [] -> :wat::core::nil nil)",
        src
    )
}

/// Eval a bool-returning `:user::compute` with body `body`, after the optional
/// sibling declarations `decls`. Returns the Value (or a stringified error —
/// at HEAD the threading bodies error in check/eval, which `.unwrap()` surfaces
/// as the disconfirmation).
fn eval_bool_with(decls: &str, body: &str) -> Result<Value, String> {
    let src = format!(
        "{decls}\n(:wat::core::defn :user::compute [] -> :wat::core::bool {body})",
    );
    let full = with_nil_main(&src);
    let world = startup_from_source(&full, None, Arc::new(InMemoryLoader::new()))
        .map_err(|e| format!("startup: {:?}", e))?;
    let ast = wat::parse_one!("(:user::compute)").map_err(|e| format!("parse: {:?}", e))?;
    let env = Environment::new();
    eval_in_frozen(&ast, &world, &env)
        .map(|tv| tv.value_owned())
        .map_err(|e| format!("eval: {:?}", e))
}

const INC: &str =
    "(:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::i64::+ x 1))";
const GT2: &str =
    "(:wat::core::fn [x <- :wat::core::i64] -> :wat::core::bool (:wat::core::i64::> x 2))";

// ═══════════════════════════════════════════════════════════════════════════
// REGRESSION — plain fn-first map, NO threading. GREEN at HEAD and after.
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn regression_fn_first_map_no_threading() {
    let body = format!("(:wat::core::= (:wat::core::map {INC} [1 2 3]) [2 3 4])");
    assert_eq!(eval_bool_with("", &body).unwrap(), Value::bool(true));
}

// ═══════════════════════════════════════════════════════════════════════════
// MINT — threading. RED at HEAD (`->`/`->>` unrecognized) → `#[ignore]`.
// ═══════════════════════════════════════════════════════════════════════════

/// `(->> [1 2 3] (map INC))` → `(map INC [1 2 3])` → [2 3 4]. Collection lands LAST.
#[test]
fn mint_thread_last_single_step() {
    let body = format!("(:wat::core::= (->> [1 2 3] (:wat::core::map {INC})) [2 3 4])");
    assert_eq!(eval_bool_with("", &body).unwrap(), Value::bool(true));
}

/// `(->> [1 2 3] (map INC) (filter GT2))` → `(filter GT2 (map INC [1 2 3]))`
/// → filter(>2) [2 3 4] → [3 4]. The two-step pipeline — the arc-247 raison d'être.
#[test]
fn mint_thread_last_pipeline() {
    let body = format!(
        "(:wat::core::= (->> [1 2 3] (:wat::core::map {INC}) (:wat::core::filter {GT2})) [3 4])"
    );
    assert_eq!(eval_bool_with("", &body).unwrap(), Value::bool(true));
}

/// `(-> 5 (i64::- 3))` → `(i64::- 5 3)` → 2. Accumulator injected FIRST.
/// Also the disambiguation proof: `->` is the return-arrow in the `:user::compute`
/// signature AND the thread-first head in the body — both in one form.
#[test]
fn mint_thread_first_injects_first() {
    let body = "(:wat::core::= (-> 5 (:wat::core::i64::- 3)) 2)";
    assert_eq!(eval_bool_with("", body).unwrap(), Value::bool(true));
}

/// `(->> 5 (i64::- 3))` → `(i64::- 3 5)` → -2. Injected LAST.
/// With the prior gate this proves thread-first ≠ thread-last (2 vs -2).
#[test]
fn mint_thread_last_injects_last() {
    let body = "(:wat::core::= (->> 5 (:wat::core::i64::- 3)) -2)";
    assert_eq!(eval_bool_with("", body).unwrap(), Value::bool(true));
}

/// Bare-symbol step: `(-> 3 :my::inc)` → `(:my::inc 3)` → 4. A non-list step is
/// wrapped into a 1-arg call of the accumulator.
#[test]
fn mint_bare_symbol_step() {
    let decls =
        "(:wat::core::defn :my::inc [x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::i64::+ x 1))";
    let body = "(:wat::core::= (-> 3 :my::inc) 4)";
    assert_eq!(eval_bool_with(decls, body).unwrap(), Value::bool(true));
}
