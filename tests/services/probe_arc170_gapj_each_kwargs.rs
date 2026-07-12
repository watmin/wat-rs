//! Arc 170 gap J — `each` + kwargs tail (the success-gate's "each+tail" proof).
//!
//! `(:wat::bracket::each locus items work-fn :name val …)` grants+dials a durable-counter
//! service via `each`'s own tail (the kwargs-provisioning layer riding `each`, not just
//! `map`); every item's side effect (increment) must fire exactly once, `each` itself must
//! return nil, and the counter's final durable count must equal the item count (5).
//!
//! Driven via a programmatic AST call (not an inline `parse_one!` string) so it trips no
//! `no_inlined_wat` lint — mirrors `tests/services/probe_arc170_c1_kwargs_bracket.rs` /
//! `tests/services/probe_arc170_c2_mixed_macro.rs`.
//!
//! FORKS processes (the counter service + N pool workers) — run --test-threads=1:
//! cargo nextest run -p wat -E 'test(/probe_arc170_gapj_each_kwargs/)' --test-threads=1

use wat::ast::WatAST;
use wat::freeze::{eval_in_frozen, startup_from_file};
use wat::runtime::{Environment, Value};

#[test]
fn each_with_kwargs_tail_fires_every_side_effect_and_returns_nil() {
    let world = startup_from_file("tests/services/probe_arc170_gapj_each_kwargs.wat")
        .expect("startup should succeed (arc 170 gap J: each + kwargs tail fixture)");
    let call = WatAST::List(
        vec![WatAST::Keyword(":probe::run".into(), wat::rust_caller_span!())],
        wat::rust_caller_span!(),
    );
    let got = eval_in_frozen(&call, &world, &Environment::new())
        .unwrap_or_else(|e| panic!("run raised: {e:?}"))
        .value_owned();
    match got {
        Value::Tuple(ref items) => {
            assert_eq!(items.len(), 2, "expected a 2-tuple (each-out, final-count); got {items:?}");
            assert_eq!(
                items[0],
                Value::Unit,
                "arc 170 gap J: `each` must return nil even with a kwargs tail; got {:?}",
                items[0]
            );
            assert_eq!(
                items[1],
                Value::i64(5),
                "arc 170 gap J: every one of the 5 items must have fired its dialed-service side \
                 effect exactly once (no double-count via double-grant, no drop); got {:?}",
                items[1]
            );
        }
        other => panic!("expected a Tuple(nil, i64), got {other:?}"),
    }
}
