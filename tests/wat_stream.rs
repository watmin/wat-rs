//! End-to-end tests for `:wat::std::stream` — the first slice of the
//! stream stdlib. Each test runs a real multi-thread pipeline through
//! `startup_from_source` + `invoke_user_main`, demonstrating the
//! idiomatic shape the trading-lab app will consume.
//!
//! Producers are passed as lambdas — user-defined wrappers like
//! `spawn-producer` accept `:fn(Sender<T>)->()` values. Keyword-path
//! coercion (so a bare `:my::producer` works the same as a lambda)
//! is a future slice; today the wrapper pattern is explicit.
//!
//! Coverage:
//!
//! - spawn-producer + collect: round-trip a finite producer.
//! - spawn-producer + map + collect: 1:1 transform composes.
//! - Three-stage pipeline with two chained maps.
//! - Empty producer terminates cleanly.
//! - for-each drives the pipeline to completion and returns :().

use wat::freeze::{invoke_user_main, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::Value;

fn run(src: &str) -> Value {
    let loader = InMemoryLoader::new();
    let world = startup_from_source(src, None, &loader).expect("startup");
    invoke_user_main(&world, Vec::new()).expect("main")
}

fn collected_i64(src: &str) -> Vec<i64> {
    match run(src) {
        Value::Vec(items) => items
            .iter()
            .map(|v| match v {
                Value::i64(n) => *n,
                other => panic!("expected i64 element; got {:?}", other),
            })
            .collect(),
        other => panic!("expected Vec; got {:?}", other),
    }
}

// ─── spawn-producer + collect ─────────────────────────────────────────

#[test]
fn spawn_producer_plus_collect_round_trips_three_values() {
    let src = r#"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)

        (:wat::core::define (:user::main -> :Vec<i64>)
          (:wat::std::stream::collect
            (:wat::std::stream::spawn-producer
              (:wat::core::lambda ((tx :rust::crossbeam_channel::Sender<i64>) -> :())
                (:wat::core::let*
                  (((_ :Option<()>) (:wat::kernel::send tx 1))
                   ((_ :Option<()>) (:wat::kernel::send tx 2))
                   ((_ :Option<()>) (:wat::kernel::send tx 3)))
                  ())))))
    "#;
    assert_eq!(collected_i64(src), vec![1, 2, 3]);
}

// ─── spawn-producer + map + collect ───────────────────────────────────

#[test]
fn spawn_producer_map_collect_doubles_each_value() {
    let src = r#"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)

        (:wat::core::define (:user::main -> :Vec<i64>)
          (:wat::core::let*
            (((source :wat::std::stream::Stream<i64>)
              (:wat::std::stream::spawn-producer
                (:wat::core::lambda ((tx :rust::crossbeam_channel::Sender<i64>) -> :())
                  (:wat::core::let*
                    (((_ :Option<()>) (:wat::kernel::send tx 1))
                     ((_ :Option<()>) (:wat::kernel::send tx 2))
                     ((_ :Option<()>) (:wat::kernel::send tx 3))
                     ((_ :Option<()>) (:wat::kernel::send tx 4)))
                    ()))))
             ((doubled :wat::std::stream::Stream<i64>)
              (:wat::std::stream::map source
                (:wat::core::lambda ((n :i64) -> :i64)
                  (:wat::core::i64::* n 2)))))
            (:wat::std::stream::collect doubled)))
    "#;
    assert_eq!(collected_i64(src), vec![2, 4, 6, 8]);
}

// ─── Three-stage pipeline ─────────────────────────────────────────────

#[test]
fn three_stage_pipeline_map_map_collect() {
    // source → map(+1) → map(*3) → collect.
    // Each stage spawns its own worker; handles carried via
    // Stream<T>'s tuple. Drop cascade flushes on termination.
    let src = r#"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)

        (:wat::core::define (:user::main -> :Vec<i64>)
          (:wat::core::let*
            (((s0 :wat::std::stream::Stream<i64>)
              (:wat::std::stream::spawn-producer
                (:wat::core::lambda ((tx :rust::crossbeam_channel::Sender<i64>) -> :())
                  (:wat::core::let*
                    (((_ :Option<()>) (:wat::kernel::send tx 0))
                     ((_ :Option<()>) (:wat::kernel::send tx 1))
                     ((_ :Option<()>) (:wat::kernel::send tx 2)))
                    ()))))
             ((s1 :wat::std::stream::Stream<i64>)
              (:wat::std::stream::map s0
                (:wat::core::lambda ((n :i64) -> :i64)
                  (:wat::core::i64::+ n 1))))
             ((s2 :wat::std::stream::Stream<i64>)
              (:wat::std::stream::map s1
                (:wat::core::lambda ((n :i64) -> :i64)
                  (:wat::core::i64::* n 3)))))
            (:wat::std::stream::collect s2)))
    "#;
    // (0+1)*3, (1+1)*3, (2+1)*3 = 3, 6, 9
    assert_eq!(collected_i64(src), vec![3, 6, 9]);
}

// ─── Empty producer terminates cleanly ────────────────────────────────

#[test]
fn empty_producer_yields_empty_collected_vec() {
    let src = r#"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)

        (:wat::core::define (:user::main -> :Vec<i64>)
          (:wat::std::stream::collect
            (:wat::std::stream::spawn-producer
              (:wat::core::lambda ((_tx :rust::crossbeam_channel::Sender<i64>) -> :())
                ()))))
    "#;
    assert_eq!(collected_i64(src), Vec::<i64>::new());
}

// ─── for-each drives to completion ────────────────────────────────────

#[test]
fn for_each_returns_unit_on_finite_producer() {
    let src = r#"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)

        (:wat::core::define (:user::main -> :())
          (:wat::std::stream::for-each
            (:wat::std::stream::spawn-producer
              (:wat::core::lambda ((tx :rust::crossbeam_channel::Sender<i64>) -> :())
                (:wat::core::let*
                  (((_ :Option<()>) (:wat::kernel::send tx 1))
                   ((_ :Option<()>) (:wat::kernel::send tx 2)))
                  ())))
            (:wat::core::lambda ((_n :i64) -> :()) ())))
    "#;
    assert!(matches!(run(src), Value::Unit));
}
