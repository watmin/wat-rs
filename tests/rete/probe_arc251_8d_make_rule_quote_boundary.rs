//! Arc 251 stone 251.8d-ii (SIXTH draw) — **the `make-rule` quote boundary reads one identity,
//! not one spelling.**
//!
//! ⛔⛔ **THE BRIEF SENT THIS DRAW TO `rete/kernel/arm.rs::compile_acc_fold`.** That address is a
//! real shape-B site and it is cured too (`src/rete/kernel/arm.rs`, `mod
//! acc_fold_head_identity_tests`) — but it is **NOT** where the fifth draw's 239 converted-floor
//! failures came from. Those are `#wat.check/ArityMismatch`, raised at FREEZE time by the type
//! checker; `compile_acc_fold` runs at FIRE time and raises `MalformedForm`. Measured, by
//! converting exactly ONE stdlib file (`wat/rete/syntax.wat`) and rebuilding:
//!
//! ```text
//!   (:wat::rete::defrule …  :when [… (?n :- (:wat::rete::acc::count) :from …)] …)
//!     → ":wat::rete::acc::count: expected 1 argument(s); got 0"
//! ```
//!
//! The mechanism: `defrule` expands to `(:wat::rete::make-rule <name> (:wat::core::quote <when>)
//! (:wat::core::quote <then>))`. Converted, its template emits **`(wat.core/quote …)`** — a
//! `WatAST::Symbol` head. `resolve::normalize`'s `make-rule` descent tested that head with a
//! KEYWORD-ONLY `matches!`, so the argument was neither recognized as a quote nor re-spelled; a
//! surviving `wat.core/quote` head then misses `check.rs::infer_list`'s keyword-only
//! `":wat::core::quote"` arm, and the quoted condition vector is **type-checked as code**. A
//! zero-argument acc-form is an arity error the moment it is read as a call.
//!
//! ⭐ **The shape is writeable BY HAND on an unconverted tree**, which is what makes this a
//! two-binary probe with no 22-minute conversion in it. Measured, `src/resolve/normalize.rs`,
//! `src/resolve/walk.rs` and `src/macros/expand.rs` reverted to `HEAD` and rebuilt:
//!
//! ```text
//!   PRE-CURE   #wat.check/ArityMismatch ":wat::rete::acc::count: expected 1 argument(s); got 0"
//!   POST-CURE  1 derived fact
//! ```
//!
//! ## The rows
//!
//! | entry | want | what it pins |
//! |---|---|---|
//! | `kw-quote` | 1 | ⛔ control — the keyword side may not move |
//! | ⭐ `sym-quote` | 1 | **THE CURE** — both quotes symbol-spelled, exactly a converted `defrule` |
//! | ⭐ `sym-when-kw-then` | 1 | isolates the `:when` door |
//! | ⭐ `kw-when-sym-then` | 1 | isolates the `:then` door — a separate read, separately cured |
//! | ⭐ `sym-quote-where-passes` | 1 | ⛔ **CODE STAYS CODE**: a `(where …)` body inside a SYMBOL-quoted `:when` is still normalized and still evaluated — the cure re-spells the MARKER, it does not freeze the argument into opaque data |
//! | `kw-quote-where-passes` | 1 | the same under the keyword spelling |
//! | ⭐ `sym-quote-where-filters` | 0 | the non-vacuity of the row above — the same body with a gate that is FALSE derives nothing, so `…-passes` cannot be passing on a `where` that never ran |
//! | `kw-quote-where-filters` | 0 | ditto, keyword |
//!
//! ⛔ **And the conditions are still DATA, proven by the cure rows themselves**: every `:when`
//! holds `(:q2::Group (?g :- :g))` and `(?n :- (…) :from (…))`, neither of which is a legal call.
//! They reach the rete validator as written. If the cure had opened the quoted region to
//! normalization or to the checker, `sym-quote` would fail exactly the way the PRE-CURE binary
//! fails — which is the failure it is built to detect.
//!
//! Run: `cargo nextest run --release -E 'test(make_rule_quote_boundary)'`

use wat::freeze::call_beside_value;
use wat::runtime::Value;

fn derived(entry: &str) -> i64 {
    match call_beside_value(file!(), entry).expect("the fixture must evaluate") {
        Value::i64(n) => n,
        other => panic!("{entry} answered {other:?}, not an i64"),
    }
}

/// ⭐ THE WHOLE TABLE IN ONE TEST — the claim is about the JOINT behaviour of one door, and a
/// per-row test would let a cure that froze every argument into data pass six of the eight.
/// Asserted as a table so a failure prints every wrong row rather than the first.
#[test]
fn the_make_rule_quote_boundary_reads_one_identity_in_both_spellings() {
    let rows: &[(&str, i64, &str)] = &[
        (":user::kw-quote", 1, "control: the keyword spelling may not move"),
        (":user::sym-quote", 1, "THE CURE: (wat.core/quote …) is the same boundary"),
        (":user::sym-when-kw-then", 1, "THE CURE: the :when door alone"),
        (":user::kw-when-sym-then", 1, "THE CURE: the :then door alone"),
        (
            ":user::sym-quote-where-passes",
            1,
            "CODE STAYS CODE: a where body inside a symbol-quoted :when still runs",
        ),
        (":user::kw-quote-where-passes", 1, "control for the row above"),
        (
            ":user::sym-quote-where-filters",
            0,
            "NON-VACUITY: the same where body with a false gate derives nothing",
        ),
        (":user::kw-quote-where-filters", 0, "control for the row above"),
    ];

    let wrong: Vec<String> = rows
        .iter()
        .filter_map(|(entry, want, why)| {
            let got = derived(entry);
            (got != *want).then(|| format!("  {entry} → {got} (want {want}) — {why}"))
        })
        .collect();

    assert!(
        wrong.is_empty(),
        "the make-rule quote boundary answered wrongly on {} row(s):\n{}",
        wrong.len(),
        wrong.join("\n")
    );
}
