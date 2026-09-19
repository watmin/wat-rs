//! Nonzero readers for census counters that the engine emits and no cost test
//! previously asserted on. A zero-expecting assert cannot tell "absent" from
//! "measured zero" — every assertion here is `> 0`.

use super::*;

use crate::rete::eval_insert::build_insert_fact;
use crate::rete::eval_test::build_test_env;
use crate::value::pmap::PMap;
use std::sync::Arc;

/// `filter:test-env-builds` / `filter:test-key-alloc` fire inside `build_test_env`,
/// which a real fire no longer calls (`exec_where` over BindSpan). Driven here
/// on the extracted function so the counters stay falsifiable.
#[test]
fn filter_test_env_counters_are_nonzero() {
    let seed = PMap::from_pairs([(
        Value::String(Arc::new("?x".into())),
        Value::i64(1),
    )]);
    let empty = Environment::new();
    let (_, rows) = super::with_count_census(|| {
        let _ = build_test_env(&seed, &empty);
    });
    let of = |name: &str| -> u64 {
        rows.iter()
            .find(|(n, _)| *n == name)
            .map(|(_, c)| *c)
            .unwrap_or(0)
    };
    assert!(
        of("filter:test-env-builds") > 0,
        "filter:test-env-builds is 0 on a direct build_test_env call"
    );
    assert!(
        of("filter:test-key-alloc") > 0,
        "filter:test-key-alloc is 0 while binding one string key"
    );
}

/// `prod:class-alloc` is the class-name String in interpreter `build_insert_fact`.
/// Compiled RHS does not take this path; a whole-fire census can read zero.
#[test]
fn prod_class_alloc_is_nonzero_on_interpreter_insert() {
    let world = freeze_src(
        "(:wat::core::defrecord :ccr::T [n <- :wat::core::i64])\n",
    );
    let ast = crate::parse_one!("(:ccr::T 1)").expect("parse fact form");
    let (_, rows) = super::with_count_census(|| {
        build_insert_fact(&ast, &PMap::new(), world.symbols())
            .expect("interpreter insert of a one-field record");
    });
    let of = |name: &str| -> u64 {
        rows.iter()
            .find(|(n, _)| *n == name)
            .map(|(_, c)| *c)
            .unwrap_or(0)
    };
    assert!(
        of("prod:class-alloc") > 0,
        "prod:class-alloc is 0 on build_insert_fact of (:ccr::T 1)"
    );
}

/// `rematch:compiled` bumps at the first statement of `exec_compiled_under`.
/// Leftover rematch on a real fire can be rare; this call is the door.
#[test]
fn rematch_compiled_is_nonzero() {
    let world = freeze_src(
        "(:wat::core::defrecord :ccr::T [n <- :wat::core::i64])\n",
    );
    let cond = crate::parse_one!("(:ccr::T (?n <- :n))").expect("parse cond");
    let compiled = crate::rete::compiled_cond::compile_alpha_ops(
        &cond,
        &["n".to_string()],
        world.symbols(),
    )
    .expect("compile bind-only cond");
    let fact = [Value::i64(1)];
    let seed = PMap::new();
    let (_, rows) = super::with_count_census(|| {
        let mut scratch = Vec::new();
        let _ = crate::rete::compiled_cond::exec_compiled_under(
            world.symbols(),
            &compiled,
            &fact,
            &mut scratch,
            &seed,
        );
    });
    let of = |name: &str| -> u64 {
        rows.iter()
            .find(|(n, _)| *n == name)
            .map(|(_, c)| *c)
            .unwrap_or(0)
    };
    assert!(
        of("rematch:compiled") > 0,
        "rematch:compiled is 0 on a direct exec_compiled_under call"
    );
}

/// Histogram buckets (`ebucket`/`tbucket` in `fire/delta.rs`) are computed names: a cost
/// test that walks them via `format!("{pfx}{suf}")` never places a literal in a reader
/// argument, so sibling READ cannot see them. Each name is passed to `of` here. A per-bucket
/// nonzero pin would freeze a distribution; the family is live via `elem-card:0`, which
/// `accum_matcher_op_census` already lists as present on this axis.
#[test]
fn binding_card_histogram_buckets_are_read() {
    let rows = accum_count_census(60, 60);
    let of = |name: &str| -> u64 {
        rows.iter()
            .find(|(n, _)| *n == name)
            .map(|(_, c)| *c)
            .unwrap_or(0)
    };
    let family = of("elem-card:0")
        + of("elem-card:1")
        + of("elem-card:2")
        + of("elem-card:3")
        + of("elem-card:4")
        + of("elem-card:5")
        + of("elem-card:6-7")
        + of("elem-card:8+")
        + of("tok-card:0")
        + of("tok-card:1")
        + of("tok-card:2")
        + of("tok-card:3")
        + of("tok-card:4")
        + of("tok-card:5")
        + of("tok-card:6-7")
        + of("tok-card:8+");
    assert!(
        family > 0,
        "the binding-card histogram counted nothing on accum 60/60"
    );
    assert!(
        of("elem-card:0") > 0,
        "elem-card:0 is 0 on accum 60/60 — the bucket the matcher-op census lists as present"
    );
}
