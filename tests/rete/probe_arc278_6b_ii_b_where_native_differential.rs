//! Arc 278 stone 6b-ii-b — `where`/TestNode in the native kernel; native `fire-rules` == `fire-rules$oracle`.
//! Dual-impl: the unprimed public Fn is native; `$oracle` is the spec mouth.
//!
//! For a rule with a `where`, both mouths agree: `(> -5 -50)` derives one ColdAndWindy; `(> -5 100)`
//! derives zero. Live mouths: `collect-rules`, `compile-all`, `insert`, `fire-rules`, `fire-rules$oracle`,
//! `query`.
//!
//! Run: cargo test --release -p wat --test probe_arc278_6b_ii_b_where_native_differential

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

/// cold-and-windy with a `where (> ?c THRESH)` filtering the joined token. The join is on ?loc
/// (Temperature ⋈ WindSpeed at the same location); the where filters by the temperature.
// rune:lint(no-inlined-wat) — world parameterized by runtime threshold (i64) — cannot be pre-extracted to a static .wat file
fn world(threshold: i64) -> String {
    format!(
        "(:wat::core::defrecord :weather::Temperature [celsius <- :wat::core::i64  location <- :wat::core::String])\n\
         (:wat::core::defrecord :weather::WindSpeed    [kph     <- :wat::core::i64  location <- :wat::core::String])\n\
         (:wat::core::defrecord :weather::ColdAndWindy [location <- :wat::core::String])\n\
         \n\
         (:wat::rete::defrule :weather::cold-and-windy\n\
           :when\n\
           [(:weather::Temperature (?loc <- :location) (?c <- :celsius))\n\
            (:weather::WindSpeed   (?loc <- :location) (?k <- :kph))\n\
            (:wat::rete::where (:wat::rete::core::i64::> ?c {threshold}))]\n\
           :then\n\
           [(:weather::ColdAndWindy :location ?loc)])\n\
         \n\
         (:wat::rete::defquery :weather::q-ColdAndWindy\n\
           :params []\n\
           :when [(:weather::ColdAndWindy)])"
    )
}

/// Fire the world through `fire_fn` (the oracle `fire-rules$oracle` or the native `fire-rules`) and count
/// the derived ColdAndWindy facts. Temperature(-5, Oslo) ⋈ WindSpeed(45, Oslo) → one joined token.
fn count(world_src: &str, fire_fn: &str) -> Result<i64, String> {
    let run = format!(
        "(:wat::core::length\n\
          (:wat::core::let\n\
            [rules   (:wat::rete::collect-rules :weather)\n\
             session (:wat::rete::compile-all rules (:wat::core::PersistentVector (:weather::q-ColdAndWindy)))\n\
             session (:wat::core::match (:wat::rete::insert session (:weather::Temperature :celsius -5 :location \"Oslo\")) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __ilimit __iused __icount) (:wat::kernel::assertion-failed! \"insert: session memory ceiling exceeded while staging\" :wat::core::None :wat::core::None)))\n\
             session (:wat::core::match (:wat::rete::insert session (:weather::WindSpeed    :kph 45 :location \"Oslo\")) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __ilimit __iused __icount) (:wat::kernel::assertion-failed! \"insert: session memory ceiling exceeded while staging\" :wat::core::None :wat::core::None)))\n\
             fired   (:wat::core::match (:wat::rete::{fire_fn} session) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! \"fire-rules: session memory ceiling exceeded\" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! \"fire-rules: fixpoint round cap exceeded\" :wat::core::None :wat::core::None)))]\n\
            (:wat::rete::query fired (:weather::q-ColdAndWindy))))"
    );
    let world = startup_from_source(world_src, Some(concat!(file!(), ":", line!())), Arc::new(InMemoryLoader::new()))
        .map_err(|e| format!("startup: {e:?}"))?;
    let ast = wat::parse_one!(&run).map_err(|e| format!("parse: {e:?}"))?;
    match eval_in_frozen(&ast, &world, &Environment::new()).map_err(|e| format!("eval: {e:?}"))?.value_owned() {
        Value::i64(n) => Ok(n),
        other => Err(format!("expected i64 count; got {other:?}")),
    }
}

/// 1 — DIFFERENTIAL, where passes: (> -5 -50) true. Native fire == oracle fire, both = 1.
#[test]
fn differential_where_passes() {
    let w = world(-50);
    let oracle = count(&w, "fire-rules$oracle").expect("oracle fire");
    let native = count(&w, "fire-rules").expect("native fire");
    assert_eq!(native, oracle, "native must equal oracle (where passes); native={native} oracle={oracle}");
    assert_eq!(native, 1, "where (> -5 -50) true → exactly one ColdAndWindy; got {native}");
}

/// 2 — DIFFERENTIAL, where blocks: (> -5 100) false. Native fire == oracle fire, both = 0.
#[test]
fn differential_where_blocks() {
    let w = world(100);
    let oracle = count(&w, "fire-rules$oracle").expect("oracle fire");
    let native = count(&w, "fire-rules").expect("native fire");
    assert_eq!(native, oracle, "native must equal oracle (where blocks); native={native} oracle={oracle}");
    assert_eq!(native, 0, "where (> -5 100) false → zero ColdAndWindy; got {native}");
}

/// 3 — the NATIVE engine alone honors the where-pass (the headline: native filters, not under-derives).
#[test]
fn native_where_passes() {
    assert_eq!(count(&world(-50), "fire-rules").expect("native fire"), 1, "native: where pass → 1");
}

/// 4 — the NATIVE engine alone honors the where-block.
#[test]
fn native_where_blocks() {
    assert_eq!(count(&world(100), "fire-rules").expect("native fire"), 0, "native: where block → 0");
}
