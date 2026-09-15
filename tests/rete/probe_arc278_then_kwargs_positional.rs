//! `:then` KWARGS IN A RUNTIME-BUILT RULE ARE READ POSITIONALLY — a silent wrong answer.
//!
//! Found 2026-08-27 by `wat-tests/rete/differential-fuzz-rules.wat` on its FIRST run, by the one
//! property that could see it.
//!
//! ## Why nothing else caught it
//!
//! The three sibling fuzzers compare ROW COUNTS. A `:then` that writes its kwargs into the wrong
//! fields derives exactly as MANY facts — with the values swapped. Count-based differentials read
//! identically on a correct engine and on one that has transposed every field it wrote.
//!
//! And an engine-vs-engine differential cannot see it either: **both engines transpose
//! identically**, so `fire-rules` and `fire-rules$oracle` agree perfectly on the wrong answer.
//! Only an independent VALUE witness catches it — here `sum(a * 1000 + b)`, asymmetric so a
//! transposition moves the number.
//!
//! ## The defect
//!
//! `reorder_kwargs_by_field_name` (`src/rete/validate.rs`) rewrites a `:then`'s kwargs into
//! declaration order — in the FREEZE-TIME `defrule` wall. A rule constructed at runtime as a
//! `Rule` VALUE never passes that wall, so its kwargs reach `build_insert_fact` in written order
//! and are consumed positionally. The two doors disagree, measured:
//!
//! | rule built by | reversed-kwargs witness | |
//! |---|---|---|
//! | declared `defrule` | **3024** | reordered — correct |
//! | runtime `Rule` value | **24003** | transposed |
//!
//! 3024 is `(a=0,b=7) (1,8) (2,9)`; 24003 is `(a=7,b=0) (8,1) (9,2)` — the same pairs, in the
//! wrong fields. This is verbatim the class arc 294's wall was built for: *"the RHS insert form
//! takes kwargs POSITIONALLY with no name-check or reorder. The 9a kwargs codemod corrupted a
//! swath of rule fixtures this way and NOTHING screamed."* The wall closed it for declarations
//! and left it open for values.
//!
//! ## Status: CLOSED 2026-08-27 — this file is now a regression gate
//!
//! Fixed at the ONE door rather than with a second reorder: `rete_kwargs_value_asts` now resolves
//! kwargs BY NAME against the type's declaration order, and every caller
//! (`build_insert_fact`, `compile_rhs`, `lower_construct`) already held those names. The
//! freeze-wall reorder that used to paper over this for declared rules is now redundant rather
//! than duplicated — which was the point: two implementations of "kwargs become positional", only
//! one of them on every path, was the defect itself.
//!
//! The agreeing control stays: declaration-order kwargs worked before and must still work.

use wat::freeze::call_beside_value;
use wat::runtime::Value;

/// `[declaration-order  reversed-order]`
fn rows() -> Vec<i64> {
    let out = call_beside_value(file!(), ":user::rows").expect("eval :user::rows");
    let items: Vec<&Value> = match &out {
        Value::wat__core__PersistentVector(v) => v.iter().collect(),
        Value::Vec(v) => v.iter().collect(),
        other => panic!("expected a vector; got {other:?}"),
    };
    let got: Vec<i64> = items
        .iter()
        .map(|v| match v {
            Value::i64(n) => *n,
            other => panic!("expected i64; got {other:?}"),
        })
        .collect();
    assert_eq!(got.len(), 2, "witness shape changed: {got:?}");
    got
}

/// CONTROL, not ignored: kwargs written in DECLARATION order already derive correctly. A fix for
/// the reversed case must not achieve agreement by breaking this one.
#[test]
fn declaration_order_kwargs_are_correct() {
    let r = rows();
    assert_eq!(
        r[0], 3024,
        "kwargs in declaration order must derive (a=x, b=y) for Src (0,7) (1,8) (2,9). Witness {r:?}"
    );
}

#[test]
fn reversed_kwargs_derive_the_same_fact_as_declaration_order() {
    let r = rows();
    assert_eq!(
        r[1], r[0],
        "writing `:b ?y :a ?x` must derive the same fact as `:a ?x :b ?y` — kwargs are NAMED. Got \
         {} vs {}; 24003 decodes as every field transposed. A declared `defrule` with these same \
         reversed kwargs derives 3024, so the freeze-time door and the runtime door disagree. \
         Witness {r:?}",
        r[1], r[0]
    );
}
