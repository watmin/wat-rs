//! Arc 278 — P12 THE EXPLAIN NORTH-STAR: "how did this fact get derived", walked back to the inputs.
//! RED at HEAD (`fire-rules-explain` / `explain` / `Why/via` are UnknownFunction); GREEN when P12 lands.
//! Contract: DESIGN-STONE-P12-explain-walk.md.
//!
//! This is the guiding light of arc 278 — the operator diagnostic: hand it a derived fact, get the why-tree
//! back to the input facts, through which rule, which gates, which supporting facts. Proven IN WAT (the walk is
//! a wat function over an explain-mode-exposed support graph; the fire is Rust).
//!
//! ## The opt-in principle (why this is a SEPARATE mode — DESIGN, R5 applied to the why-tree)
//! Diagnostics default OFF. `fire-rules'` (the public default) is the line-rate path: clears beta, no
//! provenance index. `fire-rules-explain` is opt-in: retains the support graph + records the
//! fact→producing-token index. This costs nothing and loses nothing, because the why-tree is itself a pure
//! function of `{facts, rules}` — a deferred computation. You can always re-force it: pull the stored thunk,
//! `fire-rules-explain`, walk it — bit-identical to what prod did (purity). The AWS S3-triage workflow made
//! principled.
//!
//! ## The worked surface this pins (nested `#rete/Why`, v1)
//! rune:exigere(scope-affirmative) — Out of P12's scope; DAG-sharing for fan-in is rejected
//! (first producing token only). Tracked as Out of scope in
//! docs/arc/2026/06/278-rules-engine/DESIGN-STONE-P12-explain-walk.md.
//! ```clojure
//! (:wat::rete::explain (:wat::rete::fire-rules-explain staged) (:weather::ColdAndWindy -5 40))
//! ;; → #rete/Why
//! ;;   {:fact (:weather::ColdAndWindy -5 40)
//! ;;    :rule "weather::cold-and-windy"
//! ;;    :via [ {:type :weather::Temperature :fact (:weather::Temperature -5 "Oslo") :bound {?c -5}
//! ;;            :met [(:wat::core::< -5 0)]}      ;; (< ?c 0), ?c=-5 → -5 < 0 ✓  (no :why → base fact, leaf)
//! ;;           {:type :weather::WindSpeed   :fact (:weather::WindSpeed 40 "Oslo")  :bound {?k 40}
//! ;;            :met [(:wat::core::> 40 30)]} ]}  ;; (> ?k 30), ?k=40 → 40 > 30 ✓  (base fact, leaf)
//! ;;
//! ;; a cascade level — a derived supporting fact carries a nested :why (the tree recurses to inputs):
//! (:wat::rete::explain fired (:weather::WeatherAlert -5 40))
//! ;; → #rete/Why {:fact (:weather::WeatherAlert -5 40) :rule "weather::alert"
//! ;;              :via [ {:type :weather::ColdAndWindy :fact (:weather::ColdAndWindy -5 40)
//! ;;                      :bound {?c -5, ?k 40} :met []
//! ;;                      :why #rete/Why { …the cold-and-windy tree above… }} ]}
//! ```
//! - Nodes = facts; edges = the gates carrying the conditions that fired. **`:met` is the load-bearing
//!   payload** — the rule's constraint predicates with the concrete bound values substituted in
//!   (`(< -5 0)`, `(> 40 30)`), each shown as it evaluated true. An operator reads the activation off the page
//!   without knowing the rule: *cold = `(< -5 0)`* is obvious on sight.
//! - A via-entry with no `:why` is a base/asserted fact (the leaf); a nested `:why` means the supporting fact
//!   is derived → drill in. (`Why.rule` is `Option<String>`: `None` = base; no optional field — a typed Option.)
//!
//! Run: cargo test --release -p wat --test probe_arc278_P12_explain_walk -- --include-ignored

use wat::freeze::call_beside_value;
use wat::runtime::Value;

/// LEVEL 1 — explain a directly-derived fact reaches its two input facts. `ColdAndWindy` is derived by
/// `cold-and-windy` from `Temperature` ⋈ `WindSpeed`; its why-tree's `:via` has exactly those two supporting
/// facts → length 2. Pins: `fire-rules-explain` (opt-in mode), `explain`, `Why/via`.
#[test]
fn explain_cold_and_windy_reaches_its_two_inputs() {
    let n = call_beside_value(file!(), ":user::explain-coldandwindy-via-length").expect("compute should run");
    assert!(matches!(n, Value::i64(2)), "ColdAndWindy's why-tree must reach 2 input facts (Temperature, WindSpeed); got {n:?}");
}

/// LEVEL 2 — explain a CASCADE-derived fact: `WeatherAlert` is derived by `alert` from the derived
/// `ColdAndWindy`. Its `:via` has exactly one supporting fact (the ColdAndWindy), which itself carries a
/// nested `:why` (the tree recurses). Length 1 at the top proves the cascade is walkable; the recursion to
/// inputs is the LEVEL-1 tree hanging under that single via-entry. Pins: explain over a cascade-derived fact.
#[test]
fn explain_weather_alert_has_one_derived_support() {
    let n = call_beside_value(file!(), ":user::explain-weatheralert-via-length").expect("compute should run");
    assert!(matches!(n, Value::i64(1)), "WeatherAlert's why-tree has exactly 1 supporting fact (the derived ColdAndWindy); got {n:?}");
}
