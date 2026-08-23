//! FM-2-bis PROBE-LED diagnostic for Arc 249 Stone 249.4 — can `keyword/of` and
//! `for` be reborn as WAT macros over the total-pure engine?
//!
//! PROBE-LED, not conviction-led: attempt the natural wat encoding; let the
//! substrate name the gap.
//!
//! Run: cargo nextest run --release -E 'binary(macros)' -F probe_arc249_4_rehome_in_wat --no-capture
//! (diag_first_over_vector_form / diag_keyword_to_string_over_form are excluded from the default
//! floor via `.config/nextest.toml`'s `default-filter`, not `#[ignore]` — pass `--ignore-default-filter`
//! too, or nextest intersects any `-E`/name filter with the exclusion and finds nothing to run.)

use std::sync::Arc;
use wat::freeze::startup_from_file;
use wat::runtime::{apply_function, Value};

// just-eval (rubric): each `*.wat` fixture defines a zero-arg `:user::compute`; fetch it from
// the frozen world and `apply_function` it — no inline wat driver. (Path-based rather than
// `call_beside_value` because this probe drives five distinct co-located fixtures from one `.rs`.)
fn try_eval(path: &str) -> Result<Value, String> {
    let world = startup_from_file(path).map_err(|e| format!("startup: {:?}", e))?;
    let func = world
        .symbols()
        .get(":user::compute")
        .ok_or_else(|| format!("no :user::compute in {path:?}"))?
        .clone();
    apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
        .map_err(|e| format!("eval: {:?}", e))
}

// ═══════════════════════════════════════════════════════════════════════════
// C — first/rest over a VECTOR form (#[ignore] diagnostic)
// ═══════════════════════════════════════════════════════════════════════════
// 296 Stone K, move 2, STOP-2: this diagnostic's fixture
// (probe_arc249_4_rehome_in_wat_vec_first.wat) does NOT load on the current
// runtime — `startup_from_file` raises `MalformedDefmacro` (a macro param typed
// `:wat::holon::HolonAST` instead of `:wat::WatAST`) — MEASURED, not assumed.
// That failure to load/type-check IS the gap this diagnostic exists to
// surface. `wat-scripts/`'s `every_wat_scripts_file_loads` gate parses +
// type-checks EVERY file under it on the current runtime, so moving this
// fixture there would turn that gate red — a diagnostic mangled into a shape
// it does not fit. Falling back to move 3: stays a plain `#[test]` here,
// excluded from the default floor by `.config/nextest.toml`'s
// `default-filter` (not `#[ignore]`), invoked explicitly to read the gap.
//
//   cargo nextest run --release --ignore-default-filter -E 'test(diag_first_over_vector_form)' --no-capture
#[test]
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
// 296 Stone K, move 2, STOP-2: this diagnostic's fixture
// (probe_arc249_4_rehome_in_wat_kw_to_str.wat) does NOT load on the current
// runtime — `startup_from_file` raises the same `MalformedDefmacro` shape as
// `diag_first_over_vector_form` above (macro param typed `:wat::holon::HolonAST`
// instead of `:wat::WatAST`) — MEASURED, not assumed. That failure to
// load/type-check IS the gap this diagnostic exists to surface.
// `wat-scripts/`'s `every_wat_scripts_file_loads` gate parses + type-checks
// EVERY file under it on the current runtime, so moving this fixture there
// would turn that gate red — a diagnostic mangled into a shape it does not
// fit. Falling back to move 3: stays a plain `#[test]` here, excluded from the
// default floor by `.config/nextest.toml`'s `default-filter` (not
// `#[ignore]`), invoked explicitly to read the gap.
//
//   cargo nextest run --release --ignore-default-filter -E 'test(diag_keyword_to_string_over_form)' --no-capture
#[test]
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
// D — a macro fires in TEMPLATE POSITION (nested inside another macro's quasiquote)
// ═══════════════════════════════════════════════════════════════════════════
// STONE-defservice-emits-the-binder (arc 109) — `:wat::core::keyword/of` retired (its
// whole purpose was minting the now-illegal `Head<a,b>` spelling; see `wat/core.wat`'s
// retirement note). This test's actual subject was never keyword/of's own semantics —
// it is macro-in-template-position firing, so the fixture's vehicle swapped to a local
// `:test::mk-kw` macro (see the `.wat` fixture) that exercises the identical topology
// (`:my::mk`'s quasiquote body calls `:test::mk-kw`) with no angle spelling anywhere.
#[test]
fn keyword_of_fires_in_template_position() {
    let result = try_eval("tests/macros/probe_arc249_4_rehome_in_wat_kw_of_tmpl.wat").expect("eval");
    println!("\n=== keyword_of_fires_in_template_position ===\nexpect Ok(\"foo-bar\"):\n{:#?}\n", result);
    assert_eq!(
        result,
        Value::String(Arc::new("foo-bar".to_string())),
        "a macro MUST fire in template position (inside another macro's quasiquote) \
         as a registered macro — the deleted keyword_of_inside_macro_template_with_unquote risk"
    );
}
