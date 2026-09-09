//! The quote door of `expr_ir::lower`: `:wat::rete::lower` of a quoted tree never
//! passed through expansion. `LowerCx`'s shared budget must refuse it by kind,
//! not abort.
//!
//! Source nesting is still the expander's wall (`EXPANSION_DEPTH_LIMIT`). The
//! compile-path arms are STOP-2's tripwire: lowering must not start refusing
//! programs expansion already accepts.

use std::sync::Arc;

use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::macros::EXPANSION_DEPTH_LIMIT;
use wat::runtime::{Environment, Value};

/// Nested `(:wat::rete::core::i64::+ INNER 1 :undefined -1)` towers, `depth` lists deep.
/// `inner` is the leaf (`0` for a quoted tree, `n` for a defn parameter).
fn nested_plus(depth: usize, inner: &str) -> String {
    let mut s = inner.to_string();
    for _ in 0..depth {
        s = format!("(:wat::rete::core::i64::+ {s} 1 :undefined -1)");
    }
    s
}

/// Forms the generated `defn` wrapper contributes to expansion depth, above the
/// `i64::+` tower: the defn list, its param vector, the `->` type, and the body
/// slot expansion walks to reach the tower. The compile-path wall is
/// `LIMIT - HARNESS_FORMS`. Driven: `LIMIT - 3` was already `ExpansionDepthExceeded`.
const HARNESS_FORMS: usize = 4;

// rune:lint(no-inlined-wat) — depth is the independent variable; a fixed .wat cannot sweep it.
fn quote_world(depth: usize) -> String {
    let body = nested_plus(depth, "0");
    format!(
        "\
(:wat::core::defn :user::go [] -> :wat::core::nil
  (:wat::rete::lower (:wat::core::quote {body})))
"
    )
}

// rune:lint(no-inlined-wat) — depth is the independent variable; a fixed .wat cannot sweep it.
fn source_world(depth: usize) -> String {
    let body = nested_plus(depth, "n");
    format!(
        "\
(:wat::rete::core::defn :probe::deep [n <- :wat::core::i64] -> :wat::core::i64
  {body})
"
    )
}

fn startup(src: &str) -> Result<wat::freeze::FrozenWorld, String> {
    startup_from_source(src, None, Arc::new(InMemoryLoader::new())).map_err(|e| format!("{e:?}"))
}

fn eval_go(src: &str) -> Result<Value, String> {
    let world = startup(src)?;
    let ast = wat::parse_one!("(:user::go)").map_err(|e| format!("parse go: {e:?}"))?;
    eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .map_err(|e| format!("{e:?}"))
}

#[test]
fn quote_door_accepts_a_shallow_nest() {
    match eval_go(&quote_world(8)) {
        Ok(Value::Unit) => {}
        Ok(other) => panic!("lower returns nil, got {other:?}"),
        Err(e) => panic!("shallow quoted nest must lower, got {e}"),
    }
}

#[test]
fn quote_door_refuses_past_the_bound() {
    let depth = EXPANSION_DEPTH_LIMIT + 1;
    match eval_go(&quote_world(depth)) {
        Ok(v) => panic!(
            "quoted nest of {depth} (LIMIT {EXPANSION_DEPTH_LIMIT} + 1) was ACCEPTED and returned {v:?} \
             — LowerCx has no depth budget on the quote door"
        ),
        Err(e) => {
            // rune:lint(loose-assert) — MalformedForm wraps rust_caller_span; naming the bound is the contract
            assert!(
                e.contains("EXPANSION_DEPTH_LIMIT") && e.contains("lowering nesting depth"),
                "the refusal must name the lowering depth bound, got {e}"
            );
        }
    }
}

#[test]
fn source_at_the_expansion_wall_still_compiles() {
    let depth = EXPANSION_DEPTH_LIMIT - HARNESS_FORMS;
    match startup(&source_world(depth)) {
        Ok(_) => {}
        Err(e) => panic!(
            "source nest of LIMIT - HARNESS_FORMS ({depth}) must still compile — lowering \
             refused a program expansion accepts. {e}"
        ),
    }
}

#[test]
fn source_past_the_expansion_wall_is_the_expander() {
    let depth = EXPANSION_DEPTH_LIMIT - HARNESS_FORMS + 1;
    match startup(&source_world(depth)) {
        Ok(_) => panic!(
            "source nest of LIMIT - HARNESS_FORMS + 1 ({depth}) compiled — the expander \
             wall moved or HARNESS_FORMS is wrong"
        ),
        Err(e) => {
            // rune:lint(loose-assert) — freeze error is a blob; the expander's kind name is the contract
            assert!(
                e.contains("ExpansionDepthExceeded") || e.contains("macro expansion exceeded depth"),
                "past-expansion source must be the expander's refusal, not lowering's, got {e}"
            );
            // rune:lint(loose-assert) — targeted ABSENCE: lowering must not have run
            assert!(
                !e.contains("lowering nesting depth"),
                "the expander must refuse before lowering, got {e}"
            );
        }
    }
}
