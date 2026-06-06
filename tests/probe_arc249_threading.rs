//! FM-2-bis probe for Arc 249 — threading macros `:wat::core::->` (thread-first) +
//! `:wat::core::->>` (thread-last).
//!
//! Threading is called FQDN, like every other macro in wat. The call surface is
//! `(:wat::core::-> x s1 s2)` / `(:wat::core::->> x s1 s2)` — a normal registered-macro
//! dispatch through `expand_form` → `registry.get`. No bare-symbol seam exists; the bare
//! `->` symbol is ONLY the return-arrow marker in fn signatures, not a threading head.
//!
//! `(:wat::core::->> x s1 s2)` desugars to `(s2 (s1 x))` — a left fold of the
//! accumulator through each step, injecting it as the LAST arg (`->>`) or FIRST arg
//! (`->`) of each step form. This is a pure macro-expansion-time source-to-source
//! rewrite; it desugars to ordinary nested calls BEFORE type-check, so the
//! checker/runtime never see `->` / `->>` and need no changes.
//!
//! ROW STATUS:
//!   - REGRESSION (GREEN at HEAD + after): plain fn-first `(map f xs)` — no threading.
//!     Anchors the harness + the arc-247 fn-first map the threading sits on top of.
//!   - MINT (GREEN): all 5 threading mints pass via FQDN registered-macro dispatch.
//!     Zero `#[ignore]`.
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

/// `(:wat::core::->> [1 2 3] (map INC))` → `(map INC [1 2 3])` → [2 3 4]. Collection lands LAST.
#[test]
fn mint_thread_last_single_step() {
    let body = format!("(:wat::core::= (:wat::core::->> [1 2 3] (:wat::core::map {INC})) [2 3 4])");
    assert_eq!(eval_bool_with("", &body).unwrap(), Value::bool(true));
}

/// `(:wat::core::->> [1 2 3] (map INC) (filter GT2))` → `(filter GT2 (map INC [1 2 3]))`
/// → filter(>2) [2 3 4] → [3 4]. The two-step pipeline — the arc-247 raison d'être.
#[test]
fn mint_thread_last_pipeline() {
    let body = format!(
        "(:wat::core::= (:wat::core::->> [1 2 3] (:wat::core::map {INC}) (:wat::core::filter {GT2})) [3 4])"
    );
    assert_eq!(eval_bool_with("", &body).unwrap(), Value::bool(true));
}

/// `(:wat::core::-> 5 (i64::- 3))` → `(i64::- 5 3)` → 2. Accumulator injected FIRST.
/// Threading is a normal FQDN macro (`registry.get(":wat::core::->")`) — entirely
/// distinct from the bare `->` return-arrow marker in fn signatures; no overload exists.
#[test]
fn mint_thread_first_injects_first() {
    let body = "(:wat::core::= (:wat::core::-> 5 (:wat::core::i64::- 3)) 2)";
    assert_eq!(eval_bool_with("", body).unwrap(), Value::bool(true));
}

/// `(:wat::core::->> 5 (i64::- 3))` → `(i64::- 3 5)` → -2. Injected LAST.
/// With the prior gate this proves thread-first ≠ thread-last (2 vs -2).
#[test]
fn mint_thread_last_injects_last() {
    let body = "(:wat::core::= (:wat::core::->> 5 (:wat::core::i64::- 3)) -2)";
    assert_eq!(eval_bool_with("", body).unwrap(), Value::bool(true));
}

/// Bare-keyword step: `(:wat::core::-> 3 :my::inc)` → `(:my::inc 3)` → 4. A non-list step
/// is wrapped into a 1-arg call of the accumulator.
#[test]
fn mint_bare_symbol_step() {
    let decls =
        "(:wat::core::defn :my::inc [x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::i64::+ x 1))";
    let body = "(:wat::core::= (:wat::core::-> 3 :my::inc) 4)";
    assert_eq!(eval_bool_with(decls, body).unwrap(), Value::bool(true));
}

// ═══════════════════════════════════════════════════════════════════════════
// ITEM 3 WITNESSES — empty-list-step failure shapes (arc 249 perimeter closure)
//
// `(-> x ())` — the empty step fires Option/expect AT MACRO-EXPANSION TIME
// (inside foldl running the -> body during startup). The failure is an
// uncaught `panic_any(AssertionPayload)` — startup_from_source does NOT
// catch_unwind, so the panic propagates out of startup. Caught here via
// std::panic::catch_unwind; downcast to AssertionPayload confirms the message.
//
// `(->> x ())` — the ->> empty step splices ~@() (nothing) + ~a, yielding
// `(acc)` i.e. `(5)`. Expansion and type-check both succeed (5 looks like an
// i64-returning call to the checker). Eval rejects it as MalformedForm
// (head "int" — an integer literal is not a callable head).
// ═══════════════════════════════════════════════════════════════════════════

/// `(-> x ())` — empty list step fires Option/expect at macro-expansion time.
/// The failure is a `panic_any(AssertionPayload { message: "-> step has no head" })`
/// propagating out of `startup_from_source` (which does not catch_unwind).
#[test]
fn witness_thread_first_empty_step_panics_at_expansion() {
    fn attempt() -> Result<(), String> {
        let src = r#"
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::-> 5 ()))
(:wat::core::defn :user::main [] -> :wat::core::nil nil)
"#;
        startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
            .map(|_| ())
            .map_err(|e| format!("startup: {:?}", e))
    }
    let result = std::panic::catch_unwind(attempt);
    match result {
        Err(payload) => {
            // Expected path: AssertionPayload panic from Option/expect in -> body.
            let ap = payload
                .downcast::<wat::assertion::AssertionPayload>()
                .expect("panic payload should be AssertionPayload");
            assert_eq!(
                ap.message, "-> step has no head",
                "expected '-> step has no head' panic message, got: {}",
                ap.message
            );
        }
        Ok(Ok(())) => panic!("expected panic but startup succeeded"),
        Ok(Err(e)) => panic!("expected panic but got startup Err: {}", e),
    }
}

/// `(->> x ())` — empty step desugars at expansion to `(acc)` i.e. `(5)`.
/// Startup succeeds; eval fails with MalformedForm (integer is not a callable head).
#[test]
fn witness_thread_last_empty_step_desugars_to_call_on_acc() {
    fn attempt_startup() -> Result<wat::freeze::FrozenWorld, String> {
        let src = r#"
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::->> 5 ()))
(:wat::core::defn :user::main [] -> :wat::core::nil nil)
"#;
        startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
            .map_err(|e| format!("startup: {:?}", e))
    }
    // startup_from_source must NOT panic and must return Ok (expansion+check pass).
    let world = std::panic::catch_unwind(attempt_startup)
        .expect("startup must not panic for ->> empty step")
        .expect("startup must succeed for ->> empty step");

    // Eval must fail: `(5)` has an integer head, not a callable keyword/symbol/list.
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    let env = Environment::new();
    let eval_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        eval_in_frozen(&ast, &world, &env)
    }));
    match eval_result {
        Ok(Err(e)) => {
            let msg = format!("{:?}", e);
            assert!(
                msg.contains("MalformedForm") || msg.contains("call head"),
                "expected MalformedForm eval error, got: {}",
                msg
            );
        }
        Ok(Ok(_)) => panic!("expected MalformedForm eval error but eval succeeded"),
        Err(_) => panic!("expected MalformedForm eval error but eval panicked"),
    }
}
