//! FM-2-bis PROBE-LED diagnostic for Arc 249 Stone 249.4 — can `keyword/of` and
//! `for` be reborn as WAT macros over the total-pure engine?
//!
//! PROBE-LED, not conviction-led: attempt the natural wat encoding; let the
//! substrate name the gap.
//!
//! Run: cargo nextest run --release -E 'binary(macros)' -F probe_arc249_4_rehome_in_wat -- --ignored --nocapture

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_file};
use wat::runtime::{Environment, Value};

/// Helper: try startup + eval, returning Result (never panics).
fn try_eval(path: &str) -> Result<Value, String> {
    let world = startup_from_file(path).map_err(|e| format!("startup: {:?}", e))?;
    let ast = wat::parse_one!("(:user::compute)").map_err(|e| format!("parse: {:?}", e))?;
    eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .map_err(|e| format!("eval: {:?}", e))
}

// ═══════════════════════════════════════════════════════════════════════════
// C — first/rest over a VECTOR form (#[ignore] diagnostic)
// ═══════════════════════════════════════════════════════════════════════════
#[test]
#[ignore = "249.4 diagnostic — run with --ignored to read the gap"]
fn diag_first_over_vector_form() {
    let result = try_eval("tests/macros/probe_arc249_4_rehome_in_wat_vec_first.wat");
    println!("\n=== diag_first_over_vector_form ===\nexpect Ok(10):\n{:#?}\n", result);
    let _ = result;
}

// ═══════════════════════════════════════════════════════════════════════════
// D — `for` IS REDUNDANT: canonical `~@(map ...)` reproduces it.
// ═══════════════════════════════════════════════════════════════════════════
#[test]
fn canonical_comprehension_replaces_for() {
    let result = try_eval("tests/macros/probe_arc249_4_rehome_in_wat_canon_comp.wat").expect("eval");
    println!("\n=== canonical_comprehension_replaces_for ===\nexpect Ok(11):\n{:#?}\n", result);
    assert_eq!(
        result,
        Value::i64(11),
        "the canonical `~@(map (fn [x] `tmpl) items)` MUST reproduce `for` — proving for is redundant"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// A — keyword-form → text (#[ignore] diagnostic)
// ═══════════════════════════════════════════════════════════════════════════
#[test]
#[ignore = "249.4 diagnostic — run with --ignored to read the gap"]
fn diag_keyword_to_string_over_form() {
    let result = try_eval("tests/macros/probe_arc249_4_rehome_in_wat_kw_to_str.wat");
    println!("\n=== diag_keyword_to_string_over_form ===\n{:#?}\n", result);
    let _ = result;
}

// ═══════════════════════════════════════════════════════════════════════════
// B — FULL keyword/of as a wat macro (diagnostic, non-asserting)
// ═══════════════════════════════════════════════════════════════════════════
#[test]
fn diag_keyword_of_full() {
    let result = try_eval("tests/macros/probe_arc249_4_rehome_in_wat_kw_of.wat");
    println!("\n=== diag_keyword_of_full ===\nexpect \"foo<bar,baz>\":\n{:#?}\n", result);
    // Diagnostic — read the shape; do not gate on it.
    let _ = result;
}

// ═══════════════════════════════════════════════════════════════════════════
// D — keyword/of in TEMPLATE POSITION
// ═══════════════════════════════════════════════════════════════════════════
#[test]
fn keyword_of_fires_in_template_position() {
    let result = try_eval("tests/macros/probe_arc249_4_rehome_in_wat_kw_of_tmpl.wat").expect("eval");
    println!("\n=== keyword_of_fires_in_template_position ===\nexpect Ok(\"foo<bar>\"):\n{:#?}\n", result);
    assert_eq!(
        result,
        Value::String(Arc::new("foo<bar>".to_string())),
        "keyword/of MUST fire in template position (inside another macro's quasiquote) \
         as a registered macro — the deleted keyword_of_inside_macro_template_with_unquote risk"
    );
}
