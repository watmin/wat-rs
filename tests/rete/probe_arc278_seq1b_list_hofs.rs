//! Arc 278 seq-1b — disconfirming probe: List joins the seq-HOF family. RED at HEAD.
//!
//! After seq-1a, `mappable()` (map/filter/foldl) and `ordered()` (reverse/concat) are both
//! `{Vector, PersistentVector}`-only. List supports ALL EIGHT (it is a full ordered, homogeneous,
//! variable-length sequence) but the registry gates it out and neither the runtime arms nor the checker
//! accept it. seq-1b flips `mappable()`+`ordered()` true for List and builds the eight runtime arms + the
//! checker arm.
//!
//! RED at HEAD: every op over a `List` raises (checker rejects — `extract_seq_elem` returns None for List;
//! at runtime the gate falls through to TypeMismatch). GREEN when seq-1b lands.
//!
//! Coverage: checker parity over BOTH representation surfaces (parametric `List/of` AND bare `:wat::core::List`
//! param); runtime values for all 8 ops; container-preservation (`List?` stays a List, not a Vec); an N×M
//! `concat` (two distinct lists → exact combined sum + length, not a cross-product); and the parity-guard
//! (a String reducer over an i64 List stays rejected — parity is not permissiveness).
//!
//! Run: cargo test --release -p wat --test probe_arc278_seq1b_list_hofs

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

/// Type-check a whole program at freeze time. Ok = clean; Err = a CheckError fired.
// rune:lint(no-inlined-wat) — world assembled at runtime from test-local defn strings — each test splices different HOF combinations; no static fixture covers the matrix
fn check(src: &str) -> Result<(), String> {
    startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .map(|_| ())
        .map_err(|e| format!("{e:?}"))
}

/// Build a world from one probe `defn`, start it (TYPE-CHECK fires here), then eval `call`.
// rune:lint(no-inlined-wat) — world assembled at runtime from test-local defn strings — each test splices different HOF combinations; no static fixture covers the matrix
fn eval_probe(defn: &str, call: &str) -> Result<Value, String> {
    let world = defn.to_string();
    let w = startup_from_source(&world, None, Arc::new(InMemoryLoader::new()))
        .map_err(|e| format!("startup (type-check): {e:?}"))?;
    let ast = wat::parse_one!(call).map_err(|e| format!("parse: {e:?}"))?;
    eval_in_frozen(&ast, &w, &Environment::new())
        .map_err(|e| format!("eval: {e:?}"))
        .map(|tv| tv.value_owned())
}

fn expect_i64(defn: &str, call: &str, want: i64) {
    match eval_probe(defn, call) {
        Ok(Value::i64(n)) => assert_eq!(n, want, "value: got {n} want {want}"),
        Ok(other) => panic!("expected i64({want}); got {other:?}"),
        Err(e) => panic!("List HOF should type-check + run: {e}"),
    }
}

const SUM: &str = "(:wat::core::fn [acc <- :wat::core::i64 x <- :wat::core::i64] -> :wat::core::i64 \
                     (:wat::core::i64::+ acc x))";
const DBL: &str = "(:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::i64::* x 2))";
const GT1: &str = "(:wat::core::fn [x <- :wat::core::i64] -> :wat::core::bool (:wat::core::i64::> x 1))";
const L123: &str = "(:wat::core::List/of 1 2 3)";
const MAIN: &str = "";

// ── Checker parity: all 8 ops type-check over a List (parametric AND bare param) ──

#[test]
fn list_hofs_typecheck_parametric() {
    // Each container-producing op collapsed via foldl→i64, so no container-return annotation is needed;
    // the only thing under test is List acceptance by each op. RED at HEAD.
    // Arc 118.2a: `map`/`filter`/`take`/`drop` now return a lazy `Stream`, not the original
    // container — `foldl` is container-only (Vector/PersistentVector/List), so those four
    // HOF-result folds must go through `:wat::core::reduce` (the Stream-aware clojure surface)
    // instead. `foldl` over the raw List, and `reverse`/`concat` (still eager,
    // container-preserving — unaffected by the flip) keep the original `foldl`.
    // Arc 118.B6b: `foldr` retired (it was `reverse`+`foldl` wearing a name borrowed from
    // Haskell, distinct only under laziness wat does not have) — `l-foldr` renamed
    // `l-fold-reverse`, body now `(reduce f init (reverse coll))`.
    let src = format!(
        "(:wat::core::defn :user::l-foldl  [] -> :wat::core::i64 (:wat::core::foldl {SUM} 0 {L123}))\n\
         (:wat::core::defn :user::l-fold-reverse [] -> :wat::core::i64 (:wat::core::reduce {SUM} 0 (:wat::core::reverse {L123})))\n\
         (:wat::core::defn :user::l-map    [] -> :wat::core::i64 (:wat::core::reduce {SUM} 0 (:wat::core::map {DBL} {L123})))\n\
         (:wat::core::defn :user::l-filter [] -> :wat::core::i64 (:wat::core::reduce {SUM} 0 (:wat::core::filter {GT1} {L123})))\n\
         (:wat::core::defn :user::l-rev    [] -> :wat::core::i64 (:wat::core::foldl {SUM} 0 (:wat::core::reverse {L123})))\n\
         (:wat::core::defn :user::l-take   [] -> :wat::core::i64 (:wat::core::reduce {SUM} 0 (:wat::core::take {L123} 2)))\n\
         (:wat::core::defn :user::l-drop   [] -> :wat::core::i64 (:wat::core::reduce {SUM} 0 (:wat::core::drop {L123} 1)))\n\
         (:wat::core::defn :user::l-concat [] -> :wat::core::i64 (:wat::core::foldl {SUM} 0 (:wat::core::concat {L123} {L123})))\n\
         {MAIN}"
    );
    assert!(check(&src).is_ok(), "all 8 HOFs must type-check on a List/of (parametric). Got: {:?}", check(&src));
}

#[test]
fn list_hofs_typecheck_bare_param() {
    // BARE `:wat::core::List` param (no <T>) reduces to a Path, not a Parametric — the second representation
    // surface. Must type-check through the HOFs (record fields / un-parameterized params use bare).
    // Arc 118.2a — map-bare's HOF result is a Stream; fold it via `reduce`, not `foldl`
    // (fold-bare/rev-bare are unaffected — raw List fold, and reverse stays eager).
    let src = format!(
        "(:wat::core::defn :user::fold-bare [xs <- :wat::core::List] -> :wat::core::i64 (:wat::core::foldl {SUM} 0 xs))\n\
         (:wat::core::defn :user::map-bare  [xs <- :wat::core::List] -> :wat::core::i64 (:wat::core::reduce {SUM} 0 (:wat::core::map {DBL} xs)))\n\
         (:wat::core::defn :user::rev-bare  [xs <- :wat::core::List] -> :wat::core::i64 (:wat::core::foldl {SUM} 0 (:wat::core::reverse xs)))\n\
         {MAIN}"
    );
    assert!(check(&src).is_ok(), "HOFs must type-check over a BARE List param. Got: {:?}", check(&src));
}

#[test]
fn wrong_element_rejected() {
    // GUARD — parity != permissiveness. A String reducer folded over an i64 List must be REJECTED.
    let str_sum = "(:wat::core::fn [acc <- :wat::core::String x <- :wat::core::String] -> :wat::core::String \
                     (:wat::core::string::concat acc x))";
    let src = format!("(:wat::core::defn :user::bad [] -> :wat::core::String (:wat::core::foldl {str_sum} \"\" {L123}))\n{MAIN}");
    assert!(check(&src).is_err(), "String reducer over i64 List must be rejected. Got: {:?}", check(&src));
}

// ── Runtime values: each op produces the right elements ──

// Arc 118.2a — map/filter/take/drop now return a lazy Stream; fold the HOF result via
// `reduce` (the Stream-aware clojure surface), not `foldl` (container-only). foldl
// over the raw List is unaffected (it tests foldl itself, not a HOF result).
// Arc 118.B6b — `foldr` retired; `list_foldr` renamed `list_reduce_over_reverse`, now
// spelled `(reduce f init (reverse coll))` — still List, still sums to 6.
#[test] fn list_map_sum()   { expect_i64(MAIN, &format!("(:wat::core::reduce {SUM} 0 (:wat::core::map {DBL} {L123}))"), 12); }   // 2+4+6
#[test] fn list_filter_sum(){ expect_i64(MAIN, &format!("(:wat::core::reduce {SUM} 0 (:wat::core::filter {GT1} {L123}))"), 5); }  // 2+3
#[test] fn list_foldl()     { expect_i64(MAIN, &format!("(:wat::core::foldl {SUM} 0 {L123})"), 6); }
#[test] fn list_reduce_over_reverse() { expect_i64(MAIN, &format!("(:wat::core::reduce {SUM} 0 (:wat::core::reverse {L123}))"), 6); }
#[test] fn list_take_sum()  { expect_i64(MAIN, &format!("(:wat::core::reduce {SUM} 0 (:wat::core::take {L123} 2))"), 3); }   // 1+2
#[test] fn list_drop_sum()  { expect_i64(MAIN, &format!("(:wat::core::reduce {SUM} 0 (:wat::core::drop {L123} 1))"), 5); }   // 2+3

#[test]
fn list_reverse_order() {
    // reverse (1 2 3) → (3 2 1); first element proves order, not just multiset.
    expect_i64(MAIN, &format!("(:wat::core::first (:wat::core::reverse {L123}))"), 3);
}

#[test]
fn list_concat_nxm() {
    // N×M: two DISTINCT lists (1 2 3)+(4 5) → (1 2 3 4 5). Exact combined sum 15 and length 5
    // (a cross-product would give 6 elements / sum 45).
    let cat = format!("(:wat::core::concat {L123} (:wat::core::List/of 4 5))");
    expect_i64(MAIN, &format!("(:wat::core::foldl {SUM} 0 {cat})"), 15);
    expect_i64(MAIN, &format!("(:wat::core::length {cat})"), 5);
}

// ── Container preservation: the HOFs return a List, not a Vec ──
// Proven through the CHECKER (grounded, predicate-free): a `-> :wat::core::List<i64>` return
// annotation type-checks ONLY if the op preserves the List container, and a `-> Vector<i64>`
// annotation on the same body is REJECTED. (NB: `:wat::core::List?` is the macro-engine AST-form
// predicate — `WatAST::List` — NOT a container test; container-hood is proven via types here.)

#[test]
fn list_hofs_preserve_container() {
    // Arc 118.2a: `map`/`filter`/`take`/`drop` flipped LAZY — they no longer preserve the List
    // container, they universally return a `Stream` (proven here via a `Stream<i64>` return
    // annotation, which type-checks ONLY because the op's real result type is a Stream — see
    // `list_map_is_not_vector` below for the complementary negative). `reverse`/`concat` are
    // untouched by the flip (still eager) and still preserve List — proven unchanged.
    let src = format!(
        "(:wat::core::defn :user::p-map  [] -> :wat::stream::Stream<wat::core::i64> (:wat::core::map {DBL} {L123}))\n\
         (:wat::core::defn :user::p-filt [] -> :wat::stream::Stream<wat::core::i64> (:wat::core::filter {GT1} {L123}))\n\
         (:wat::core::defn :user::p-rev  [] -> :wat::core::List<wat::core::i64> (:wat::core::reverse {L123}))\n\
         (:wat::core::defn :user::p-take [] -> :wat::stream::Stream<wat::core::i64> (:wat::core::take {L123} 2))\n\
         (:wat::core::defn :user::p-drop [] -> :wat::stream::Stream<wat::core::i64> (:wat::core::drop {L123} 1))\n\
         (:wat::core::defn :user::p-cat  [] -> :wat::core::List<wat::core::i64> (:wat::core::concat {L123} {L123}))\n\
         {MAIN}"
    );
    assert!(check(&src).is_ok(), "map/filter/take/drop must yield Stream<i64> (arc 118.2a); reverse/concat must still preserve List<i64>. Got: {:?}", check(&src));
}

#[test]
fn list_map_is_not_vector() {
    // NEGATIVE: map over a List yields a List, NOT a Vector — a Vector<i64> annotation must be REJECTED.
    let src = format!(
        "(:wat::core::defn :user::wrong [] -> :wat::core::Vector<wat::core::i64> (:wat::core::map {DBL} {L123}))\n{MAIN}"
    );
    assert!(check(&src).is_err(), "map over a List must NOT satisfy a Vector return (preservation, not coercion). Got: {:?}", check(&src));
}
