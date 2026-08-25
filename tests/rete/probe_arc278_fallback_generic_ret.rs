//! Arc 278 — a fallback op's undefined point is decided by its DECLARED `ret`,
//! not by sniffing the runtime value.
//!
//! THE DIVERGENCE THIS CLOSES, measured on this fixture 2026-08-24:
//!
//!     native, sniffing the value   -> 1   (fallback -1.0 taken; the rule FIRED)
//!     the $oracle                  -> 0
//!     native, guarding on ret      -> 0   (the element +Inf returned)
//!
//! The fast path disagreed with the engine's own definition of correct. It was
//! raised by a `solvere` cast as an L2 "structural mumble" — three hand-written
//! copies of one classification, of which only `runtime.rs`'s guarded on the row's
//! declared return type. Grounding it turned an L2 into a live L1: the two rete
//! copies (`where_tree.rs`'s `exec_dim`, `expr_ir.rs`'s `exec`) sniffed, and for a
//! generic-`ret` row they answer differently from both `runtime.rs` and the oracle.
//!
//! WHY NO EXISTING FIXTURE COULD SHOW IT. The `where-*` corpus exercises the f64
//! ARITHMETIC family, whose rows declare `ret: F64` — exactly the case where sniff
//! and guard agree. The rows where they differ are the six generic ones
//! (`get`/`first` over PersistentVector/Vector/List, `ret: Var("T")`), and nothing
//! drove a non-finite float through them. `runtime.rs`'s own comment predicted this
//! precisely — "a value-sniff would silently change behaviour for any future row
//! that happens to return a float for a non-arithmetic reason" — and the future row
//! already existed, six of them, in the same table.
//!
//! `first` returning -1.0 where the element is +Inf is not a rounding difference. It
//! is the WRONG ELEMENT, silently, from a total op whose whole purpose is to be
//! predictable.
//!
//! NO CLARA TWIN, deliberately: `:undefined` is a wat-specific totality mechanism
//! with no Clara equivalent, so this lives in `tests/rete/` rather than the
//! `where-*` family (whose harness requires a `.clj` twin for every member). The two
//! implementations that HAVE the concept are both asserted here.
//!
//! Run: cargo test --release -p wat --test rete probe_arc278_fallback_generic_ret

use wat::freeze::call_beside_value;
use wat::runtime::Value;

fn native_and_oracle() -> (i64, i64) {
    let out = call_beside_value(file!(), ":user::native-and-oracle")
        .expect("fixture should fire cleanly on both engines");
    let items: Vec<&Value> = match &out {
        Value::wat__core__PersistentVector(v) => v.iter().collect(),
        Value::Vec(v) => v.iter().collect(),
        other => panic!("expected a vector; got {other:?}"),
    };
    assert_eq!(items.len(), 2, "witness shape changed: {items:?}");
    let n = |v: &Value| match v {
        Value::i64(x) => *x,
        other => panic!("expected i64; got {other:?}"),
    };
    (n(items[0]), n(items[1]))
}

#[test]
fn a_generic_ret_row_does_not_treat_a_non_finite_element_as_its_undefined_point() {
    let (native, _) = native_and_oracle();
    assert_eq!(
        native, 0,
        "`PersistentVector/first` declares `ret: Var(\"T\")`, so a non-finite element is \
         NOT its undefined point — the element (+Inf) must come back, and the rule must \
         not fire. Getting 1 means the fallback (-1.0) was substituted for the real \
         element: the classification is sniffing the runtime value instead of guarding \
         on the row's declared `ret`."
    );
}

#[test]
fn native_agrees_with_the_oracle_on_the_generic_ret_row() {
    let (native, oracle) = native_and_oracle();
    assert_eq!(
        native, oracle,
        "native and the $oracle disagree on where a Fallback-class op's undefined point \
         is. This exact shape diverged before 2026-08-24 (native 1, oracle 0) and no \
         fixture in the where-* corpus could reach it, because that corpus exercises the \
         f64 arithmetic family where the two spellings happen to agree."
    );
}
