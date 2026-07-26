//! PROVES the claim: the macros-emit-macros hoist is depth-UNBOUNDED, not just "works for macros³".
//!
//! `is_do_containing_defmacro` + `hoist_defmacros_from_do` recurse through nested `do`s with no depth
//! cap, so a `defmacro` born any number of macro-emission hops deep still registers. This probe
//! isolates exactly that — a generator macro emits a `defmacro` buried FOUR `do`s deep (deeper than
//! the kwargs/defservice case), with NO kwargs and NO hygiene interaction. If the deeply-nested macro
//! registers and is callable, the recursion is proven unbounded (4 is arbitrary; the recursion has no
//! cap — N=1, N=4, N=∞ are the same code path).
//!
//! Wat source lives in the co-located fixture: probe_macros_unbounded_depth.wat
//! (slurped via startup_beside(file!())).
//!
//! Run: cargo test --release -p wat --test probe_macros_unbounded_depth

use wat::freeze::call_beside_value;
use wat::runtime::Value;

// just-eval (rubric): the probe is a zero-arg entry fn in the co-located fixture, driven via
// call_beside_value — no inline wat driver expression.
#[test]
fn defmacro_buried_four_dos_deep_still_hoists_and_is_callable() {
    let got = call_beside_value(file!(), ":t::use-deep")
        .unwrap_or_else(|e| panic!("(:t::use-deep) raised: {e:?}"));
    assert!(
        matches!(got, Value::i64(42)),
        "expected 42 — a defmacro 4 do-levels deep must register; got {got:?}"
    );
}
