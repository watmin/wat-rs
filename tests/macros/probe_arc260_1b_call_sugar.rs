//! Arc 260.1b — the CALL-SUGAR side: a `& [argspec]` kwargs fn is callable with inline `:k v` and
//! `{map}` (the companion-macro path; NOT the abandoned check/eval plain-fn reorder).
//!
//! `defn` with a kwargs section emits a COMPANION MACRO under the fn's name + the positional impl
//! under `<name>$impl` (the `$` = apparatus-minted-internal sigil — Clojure-faithful: `clojure.core$map`).
//! The macro scoops the trailing `:k v` pairs / `{map}` literal → reorders by the `::Kwargs` field order
//! → `(<name>$impl pos-args (::Kwargs …))`. The explicit-record form (260.1a) stays valid (passthrough).
//!
//! All four call forms must yield 444 (443 + (tls?1:0)):
//!   - inline `:k v` in order            (connect "h" :port 443 :tls true)
//!   - inline `:k v` OUT OF ORDER        (connect "h" :tls true :port 443)   ← only a real reorder gets this
//!   - literal `{map}`                   (connect "h" {:port 443 :tls true})
//!   - explicit record (escape hatch)    (connect "h" (connect::Kwargs 443 true))
//!
//! Disconfirmer: a field declared as PascalCase (FooBar) must be matchable
//! by `:foo-bar` in the call site.
//!
//! Wat source lives in the co-located fixture: probe_arc260_1b_call_sugar.wat
//! (slurped via startup_beside(file!())).
//!
//! Run: cargo test --release -p wat --test probe_arc260_1b_call_sugar

use wat::freeze::call_beside_value;
use wat::runtime::Value;

// just-eval (rubric): each call form is a zero-arg entry fn in the co-located fixture, driven via
// call_beside_value — no inline wat driver expression.

#[test]
fn kwargs_call_sugar_kv_map_and_record_all_agree() {
    for f in [":user::via-kv", ":user::via-kv-reorder", ":user::via-map", ":user::via-record"] {
        let got = call_beside_value(file!(), f)
            .unwrap_or_else(|e| panic!("({f}) raised: {e:?}"));
        assert!(
            matches!(got, Value::i64(444)),
            "expected 444 from ({f}): :k v / {{map}} / record all lower to (connect$impl \"h\" \
             (connect::Kwargs 443 true)) → 443 + 1; got {got:?}"
        );
    }
}

/// Disconfirmer: `:foo-bar` in the call site matches a field declared as `FooBar`.
/// Proves the pascal->kebab-in matching path (arc 265 registry). A naive
/// string-compare `:foo-bar` vs `FooBar` would fail; the correct path converts
/// `FooBar` → `"foo-bar"` and strips `:` from `:foo-bar` → `"foo-bar"`.
#[test]
fn pascal_case_field_matched_by_kebab_call_key() {
    // via-kv-pascal calls (:user::pascal-fn :foo-bar 42) — lowered by companion macro at startup
    let got = call_beside_value(file!(), ":user::via-kv-pascal").expect("startup should succeed");
    assert!(
        matches!(got, Value::i64(42)),
        "expected 42: :foo-bar should match FooBar via pascal->kebab-in; got {got:?}"
    );
    // via-map-pascal calls (:user::pascal-fn {{:foo-bar 99}}) — map literal path
    let got_map = call_beside_value(file!(), ":user::via-map-pascal").expect("startup should succeed");
    assert!(
        matches!(got_map, Value::i64(99)),
        "expected 99 via map literal {{:foo-bar 99}}; got {got_map:?}"
    );
}
