//! Arc 278 stone P12c — the EXPLAIN payload (`:constraints` / `:bindings` / `:pattern` / `rule`).
//!
//! Live mouths: `fire-rules-explain`, `explain`, `DerivationNode/via`, `DerivationNode/rule`,
//! `DerivationStep/pattern`, `DerivationStep/bindings`, `DerivationStep/constraints`. Each support
//! edge (`DerivationStep`) carries the satisfied constraint with concrete values substituted
//! (`(:wat::core::< -5 0)`), the per-step bound vars, the matched type, and the node's rule.
//! `via[0]` is the Temperature step: pattern `weather::Temperature`, `?c = -5`, one constraint.
//! Root rule is Some("weather::cold-and-windy"); a base fact's rule is None.
//!
//! Run: cargo test --release -p wat --test probe_arc278_P12c_explain_payload -- --include-ignored

use wat::ast::WatAST;
use wat::freeze::call_beside_value;
use wat::runtime::Value;

/// Invoke a co-located zero-arg entry (each rebuilds the shared lifecycle prefix — `root` /
/// `step0` — internally, then applies its own tail).
fn nav(fn_name: &str) -> Value {
    call_beside_value(file!(), fn_name).expect("compute should run")
}

/// The shared prefix yields a DerivationNode whose via is non-empty (Temperature ⋈ WindSpeed).
#[test]
fn explain_cw_root_has_nonempty_via() {
    let v = nav(":user::explain-cw-via-length");
    assert!(matches!(v, Value::i64(n) if n > 0), "explain-cw-root via must be non-empty; got {v:?}");
}

/// PATTERN — the first step matched a Temperature condition.
#[test]
fn step_pattern_is_the_matched_type() {
    let v = nav(":user::step-pattern");
    assert!(matches!(&v, Value::String(s) if s.as_str() == "weather::Temperature"), "pattern = matched type; got {v:?}");
}

/// BINDINGS — per-step: the Temperature step bound ?c = -5 (projected to THIS condition's vars).
#[test]
fn step_bindings_are_per_step() {
    let v = nav(":user::step-bindings-c");
    assert!(matches!(&v, Value::Option(o) if matches!(&**o, Some(Value::i64(-5)))), "bindings[?c] = -5; got {v:?}");
}

/// RULE (Some) — the root (a derived fact) carries its rule name.
#[test]
fn derived_node_rule_is_some() {
    let v = nav(":user::derived-node-rule");
    assert!(
        matches!(&v, Value::Option(o) if matches!(&**o, Some(Value::String(s)) if s.as_str() == "weather::cold-and-windy")),
        "root rule = Some(\"weather::cold-and-windy\"); got {v:?}"
    );
}

/// RULE (None) — a base/asserted supporting fact has no rule (renders nil).
#[test]
fn base_node_rule_is_none() {
    let v = nav(":user::base-node-rule");
    assert!(matches!(&v, Value::Option(o) if o.is_none()), "base fact rule = None; got {v:?}");
}

/// CONSTRAINTS count — one satisfied predicate on the Temperature step ((< ?c 0)).
#[test]
fn step_has_one_constraint() {
    let v = nav(":user::step-constraints-count");
    assert!(matches!(v, Value::i64(1)), "one constraint on the Temperature step; got {v:?}");
}

/// CONSTRAINTS substitution (THE load-bearing assertion) — the satisfied predicate is the form with the bound
/// value substituted: `(:wat::core::< -5 0)` (?c → -5), NOT `(:wat::core::< ?c 0)`. Span-agnostic structural match.
#[test]
fn constraint_is_the_substituted_form() {
    let v = nav(":user::step-constraint-0");
    let Value::wat__WatAST(a) = &v else { panic!("constraint must be a WatAST form; got {v:?}") };
    let WatAST::List(items, _) = a.as_ref() else { panic!("constraint must be a list form; got {a:?}") };
    // rune:lint(no-inlined-wat) — "(op a b)" here is prose shorthand in an assert-failure
    // message (describing the expected 3-item shape), not wat source; it happens to
    // round-trip through the reader as a trivial bare-symbol list, but it is never evaluated.
    assert_eq!(items.len(), 3, "(op a b); got {items:?}");
    assert!(matches!(items[1], WatAST::IntLit(-5, _)), "operand a must be the substituted -5 (not ?c); got {:?}", items[1]);
    assert!(matches!(items[2], WatAST::IntLit(0, _)), "operand b must be 0; got {:?}", items[2]);
}
