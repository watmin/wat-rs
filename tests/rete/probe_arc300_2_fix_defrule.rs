//! Arc 300.2 probe — the fix conversion as rete defrules.
//!
//! Explores whether rete defrules can express the faithful-Clojure conversion.
//!
//! ## Findings
//!
//! ### What works (conditions + arrow→:-)
//!
//! The rete CONDITION language — alpha-match inline constraints plus `:where` TestNodes —
//! can express ALL required predicates:
//!   - inline `(:wat::core::= ?kind "symbol")`: equality constraint in the fact-type clause.
//!   - `(:wat::rete::where (:wat::core::or ...))`:`or` over bound ?vars.
//!   - `(:wat::rete::where (:fix::head-keyword-str? ?name))`: user-defined string predicate.
//!   - `(:wat::rete::where (:wat::core::not ?post-arrow))`: bool negation on a bound ?var.
//!
//! The `arrow→:-` rule fires end-to-end: conditions bind ?offset/?len/?name, the :where
//! filters to arrow symbols, and the RHS produces Edit(offset, len, ":-") via a StringLit
//! that `resolve_operand` handles directly.
//!
//! ### STOP-1: head-keyword→symbol (and keyword→type-form)
//!
//! `build_insert_fact` resolves RHS fact-form args via `resolve_operand`, which supports:
//!   ?var  → look up in token bindings (returns the bound Value)
//!   :field → read from the current fact's fields (empty in RHS — always None)
//!   literal (IntLit/FloatLit/BoolLit/StringLit) → its bare Value
//!   ANYTHING ELSE → None → RuntimeError at fire time
//!
//! Therefore `(:fix::Edit ?offset ?len (:wat::core::keyword/to-symbol ?name))` in the :then
//! would raise at fire time: the nested list `(:wat::core::keyword/to-symbol ?name)` is
//! not a supported operand. (Even if it were evaluated, `keyword/to-symbol` requires a
//! `WatAST::Keyword` node, but `?name` binds a `Value::String`.)
//!
//! The only available text for the :then is `?name` — which binds the raw keyword string
//! ":wat::core::defrecord". That is wrong: the golden wants "wat.core/defrecord".
//!
//! The STOP-1 is at the RHS: the v1 `build_insert_fact` cannot evaluate function calls;
//! the conversion text (`keyword/to-symbol` result) cannot be produced in the :then.
//!
//! ### Rete expressiveness verdict
//!
//! - `arrow→:-`: EXPRESSIBLE (literal ":-" in :then). One rule fully proven.
//! - `head-keyword→symbol`: CONDITIONS expressible; RHS STOP-1.
//! - `keyword→type-form`: same STOP-1 (same RHS limitation).
//!
//! To achieve byte-identical golden output via the rete engine, the fact model would need
//! to carry precomputed converted text (computed in the driver where AST nodes are
//! accessible), so the rules dispatch on conditions and pass through the precomputed text.
//! That design works but shifts the conversion logic from the rules into the driver.
//!
//! Run: cargo test --release -p wat --test probe_arc300_2_fix_defrule

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

fn ev(world: &wat::freeze::FrozenWorld, expr: &str) -> Value {
    eval_in_frozen(
        &wat::parse_one!(expr).expect("parse"),
        world,
        &Environment::new(),
    )
    .unwrap_or_else(|e| panic!("eval raised: {e:?}"))
    .value_owned()
}

// ── (1) arrow→:- rule — fully expressible ───────────────────────────────────

#[test]
fn arrow_rule_fires_one_edit() {
    let world = startup_beside(file!()).expect("world freezes with fact model + rules");

    // Assert a Node for "<-" (symbol, arrow) at offset=0 len=2 post-arrow=false.
    let count = ev(
        &world,
        r#"(:wat::core::let
             [rules   (:wat::rete::collect-rules :fix)
              session (:wat::rete::compile rules)
              session (:wat::rete::insert session (:fix::Node "symbol" "<-" 0 2 false))
              fired   (:wat::rete::fire-rules session)]
             (:wat::core::length (:wat::rete::query fired :fix::Edit)))"#,
    );
    assert_eq!(count, Value::i64(1), "arrow→:- derives one Edit; got {count:?}");
}

#[test]
fn arrow_rule_edit_text_is_colon_bind() {
    let world = startup_beside(file!()).expect("world freezes");

    // The derived Edit's text must be ":-" (the literal from the :then RHS).
    let text = ev(
        &world,
        r#"(:wat::core::let
             [rules   (:wat::rete::collect-rules :fix)
              session (:wat::rete::compile rules)
              session (:wat::rete::insert session (:fix::Node "symbol" "<-" 0 2 false))
              fired   (:wat::rete::fire-rules session)
              edits   (:wat::rete::query fired :fix::Edit)
              edit    (:wat::core::first edits)]
             (:fix::Edit/text edit))"#,
    );
    assert_eq!(
        text,
        Value::String(Arc::new(":-".to_string())),
        "arrow→:- produces the literal ':-' in :then; got {text:?}"
    );
}

#[test]
fn arrow_rule_right_arrow_also_fires() {
    let world = startup_beside(file!()).expect("world freezes");

    // "->" also triggers the arrow→:- rule (the :where uses or).
    let count = ev(
        &world,
        r#"(:wat::core::let
             [rules   (:wat::rete::collect-rules :fix)
              session (:wat::rete::compile rules)
              session (:wat::rete::insert session (:fix::Node "symbol" "->" 5 2 false))
              fired   (:wat::rete::fire-rules session)]
             (:wat::core::length (:wat::rete::query fired :fix::Edit)))"#,
    );
    assert_eq!(count, Value::i64(1), "'->' also triggers arrow rule; got {count:?}");
}

#[test]
fn non_arrow_symbol_no_edit() {
    let world = startup_beside(file!()).expect("world freezes");

    // A non-arrow symbol (e.g. "path") should produce zero Edits.
    let count = ev(
        &world,
        r#"(:wat::core::let
             [rules   (:wat::rete::collect-rules :fix)
              session (:wat::rete::compile rules)
              session (:wat::rete::insert session (:fix::Node "symbol" "path" 10 4 false))
              fired   (:wat::rete::fire-rules session)]
             (:wat::core::length (:wat::rete::query fired :fix::Edit)))"#,
    );
    assert_eq!(count, Value::i64(0), "non-arrow symbol produces no Edit; got {count:?}");
}

// ── (2) STOP-1: head-keyword→symbol conditions fire but text is wrong ────────
//
// These tests DOCUMENT the STOP-1 gap rather than assert the desired behavior.
// The conditions ARE expressible (the rule fires). The RHS CANNOT produce the
// converted text — it can only bind ?name which is the raw ":wat::core::defrecord"
// string, not the desired "wat.core/defrecord".

#[test]
fn head_keyword_rule_conditions_fire() {
    // Proves: conditions (inline = constraint + :where string predicates) ARE expressible.
    // The rule fires for a head-keyword Node.
    let world = startup_beside(file!()).expect("world freezes");

    let count = ev(
        &world,
        r#"(:wat::core::let
             [rules   (:wat::rete::collect-rules :fix)
              session (:wat::rete::compile rules)
              session (:wat::rete::insert session (:fix::Node "keyword" ":wat::core::defrecord" 1 21 false))
              fired   (:wat::rete::fire-rules session)]
             (:wat::core::length (:wat::rete::query fired :fix::Edit)))"#,
    );
    // One Edit derives — the conditions fired.  But the text is wrong (see below).
    assert_eq!(count, Value::i64(1), "conditions fire for head-keyword Node; got {count:?}");
}

#[test]
fn head_keyword_stop1_text_is_raw_name_not_converted() {
    // STOP-1 EVIDENCE: the derived Edit's text is the raw ?name binding ":wat::core::defrecord"
    // NOT the converted "wat.core/defrecord" the golden requires.
    //
    // Root cause: build_insert_fact calls resolve_operand for each :then fact-form arg.
    // resolve_operand only handles ?var / :field / literals.  The desired expression
    // (:wat::core::keyword/to-symbol ?name) is a nested List → resolve_operand returns
    // None → RuntimeError.  So the rule's :then uses ?name (raw String) as text instead.
    //
    // This test PASSES — it asserts the observed (wrong) behavior, confirming the gap.
    let world = startup_beside(file!()).expect("world freezes");

    let text = ev(
        &world,
        r#"(:wat::core::let
             [rules   (:wat::rete::collect-rules :fix)
              session (:wat::rete::compile rules)
              session (:wat::rete::insert session (:fix::Node "keyword" ":wat::core::defrecord" 1 21 false))
              fired   (:wat::rete::fire-rules session)
              edits   (:wat::rete::query fired :fix::Edit)
              edit    (:wat::core::first edits)]
             (:fix::Edit/text edit))"#,
    );
    // STOP-1: raw name, not "wat.core/defrecord" (what the golden requires).
    assert_eq!(
        text,
        Value::String(Arc::new(":wat::core::defrecord".to_string())),
        "STOP-1: text is raw name not converted symbol; got {text:?}"
    );
}

#[test]
fn post_arrow_keyword_no_head_keyword_edit() {
    // A post-arrow keyword (post-arrow=true) is NOT matched by head-keyword→symbol.
    // The :where (:wat::core::not ?post-arrow) guard correctly excludes it.
    // (It would be matched by keyword→type-form — not yet written due to STOP-1.)
    let world = startup_beside(file!()).expect("world freezes");

    let count = ev(
        &world,
        r#"(:wat::core::let
             [rules   (:wat::rete::collect-rules :fix)
              session (:wat::rete::compile rules)
              session (:wat::rete::insert session (:fix::Node "keyword" ":wat::core::String" 10 18 true))
              fired   (:wat::rete::fire-rules session)]
             (:wat::core::length (:wat::rete::query fired :fix::Edit)))"#,
    );
    // head-keyword→symbol does NOT fire for post-arrow keywords.
    // No other rule covers this case (keyword→type-form not written — same STOP-1).
    assert_eq!(count, Value::i64(0), "post-arrow keyword excluded from head-keyword rule; got {count:?}");
}
