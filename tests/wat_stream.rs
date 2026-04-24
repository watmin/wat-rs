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

use std::sync::Arc;
use wat::freeze::{invoke_user_main, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::Value;

fn run(src: &str) -> Value {
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new())).expect("startup");
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

// ─── from-receiver ───────────────────────────────────────────────────

#[test]
fn from_receiver_wraps_raw_queue_into_stream() {
    // Caller manages their own queue + spawn, then hands the pair
    // to from-receiver to plug into the stream stdlib.
    //
    // The setup (make-queue, spawn, from-receiver) lives in a helper
    // define so `tx` and `pair` drop when the helper returns. Only
    // the returned Stream<T> (holding rx + handle) survives into
    // main. This is the same scope-IS-shutdown discipline that
    // forced take to be a stage — if main held tx across collect,
    // collect would wait forever on a never-closing channel.
    let src = r#"

        (:wat::core::define (:test::build-stream -> :wat::std::stream::Stream<i64>)
          (:wat::core::let*
            (((pair :(rust::crossbeam_channel::Sender<i64>,rust::crossbeam_channel::Receiver<i64>))
              (:wat::kernel::make-bounded-queue :i64 1))
             ((tx :rust::crossbeam_channel::Sender<i64>) (:wat::core::first pair))
             ((rx :rust::crossbeam_channel::Receiver<i64>) (:wat::core::second pair))
             ((handle :wat::kernel::ProgramHandle<()>)
              (:wat::kernel::spawn
                (:wat::core::lambda ((s :rust::crossbeam_channel::Sender<i64>) -> :())
                  (:wat::core::let*
                    (((_ :Option<()>) (:wat::kernel::send s 10))
                     ((_ :Option<()>) (:wat::kernel::send s 20))
                     ((_ :Option<()>) (:wat::kernel::send s 30)))
                    ()))
                tx)))
            (:wat::std::stream::from-receiver rx handle)))

        (:wat::core::define (:user::main -> :Vec<i64>)
          (:wat::std::stream::collect (:test::build-stream)))
    "#;
    assert_eq!(collected_i64(src), vec![10, 20, 30]);
}

#[test]
fn from_receiver_composes_with_map() {
    // from-receiver stream feeds into a map stage, then collect.
    // Same helper-define pattern so tx drops before collect runs.
    let src = r#"

        (:wat::core::define (:test::build-stream -> :wat::std::stream::Stream<i64>)
          (:wat::core::let*
            (((pair :(rust::crossbeam_channel::Sender<i64>,rust::crossbeam_channel::Receiver<i64>))
              (:wat::kernel::make-bounded-queue :i64 1))
             ((tx :rust::crossbeam_channel::Sender<i64>) (:wat::core::first pair))
             ((rx :rust::crossbeam_channel::Receiver<i64>) (:wat::core::second pair))
             ((handle :wat::kernel::ProgramHandle<()>)
              (:wat::kernel::spawn
                (:wat::core::lambda ((s :rust::crossbeam_channel::Sender<i64>) -> :())
                  (:wat::core::let*
                    (((_ :Option<()>) (:wat::kernel::send s 1))
                     ((_ :Option<()>) (:wat::kernel::send s 2))
                     ((_ :Option<()>) (:wat::kernel::send s 3)))
                    ()))
                tx)))
            (:wat::std::stream::from-receiver rx handle)))

        (:wat::core::define (:user::main -> :Vec<i64>)
          (:wat::core::let*
            (((source :wat::std::stream::Stream<i64>) (:test::build-stream))
             ((doubled :wat::std::stream::Stream<i64>)
              (:wat::std::stream::map source
                (:wat::core::lambda ((n :i64) -> :i64)
                  (:wat::core::i64::* n 2)))))
            (:wat::std::stream::collect doubled)))
    "#;
    assert_eq!(collected_i64(src), vec![2, 4, 6]);
}

// ─── spawn-producer + collect ─────────────────────────────────────────

#[test]
fn spawn_producer_plus_collect_round_trips_three_values() {
    let src = r#"

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

// ─── filter ──────────────────────────────────────────────────────────

#[test]
fn filter_keeps_only_passing_values() {
    // 1..=6, keep evens → [2, 4, 6].
    let src = r#"

        (:wat::core::define (:user::main -> :Vec<i64>)
          (:wat::core::let*
            (((source :wat::std::stream::Stream<i64>)
              (:wat::std::stream::spawn-producer
                (:wat::core::lambda ((tx :rust::crossbeam_channel::Sender<i64>) -> :())
                  (:wat::core::let*
                    (((_ :Option<()>) (:wat::kernel::send tx 1))
                     ((_ :Option<()>) (:wat::kernel::send tx 2))
                     ((_ :Option<()>) (:wat::kernel::send tx 3))
                     ((_ :Option<()>) (:wat::kernel::send tx 4))
                     ((_ :Option<()>) (:wat::kernel::send tx 5))
                     ((_ :Option<()>) (:wat::kernel::send tx 6)))
                    ()))))
             ((evens :wat::std::stream::Stream<i64>)
              (:wat::std::stream::filter source
                (:wat::core::lambda ((n :i64) -> :bool)
                  (:wat::core::= (:wat::core::i64::/ (:wat::core::i64::* n 2) 2)
                                 n)))))
            (:wat::std::stream::collect evens)))
    "#;
    // Identity check inside the lambda — (n*2)/2 == n is always true.
    // Swap in a real parity check:
    let src = src.replace(
        "(:wat::core::= (:wat::core::i64::/ (:wat::core::i64::* n 2) 2)\n                                 n)",
        "(:wat::core::= (:wat::core::i64::* (:wat::core::i64::/ n 2) 2) n)",
    );
    assert_eq!(collected_i64(&src), vec![2, 4, 6]);
}

// ─── fold ────────────────────────────────────────────────────────────

#[test]
fn fold_sums_the_stream() {
    let src = r#"

        (:wat::core::define (:user::main -> :i64)
          (:wat::std::stream::fold
            (:wat::std::stream::spawn-producer
              (:wat::core::lambda ((tx :rust::crossbeam_channel::Sender<i64>) -> :())
                (:wat::core::let*
                  (((_ :Option<()>) (:wat::kernel::send tx 10))
                   ((_ :Option<()>) (:wat::kernel::send tx 20))
                   ((_ :Option<()>) (:wat::kernel::send tx 30)))
                  ())))
            0
            (:wat::core::lambda ((acc :i64) (x :i64) -> :i64)
              (:wat::core::i64::+ acc x))))
    "#;
    assert!(matches!(run(src), Value::i64(60)));
}

#[test]
fn fold_with_empty_stream_returns_init() {
    let src = r#"

        (:wat::core::define (:user::main -> :i64)
          (:wat::std::stream::fold
            (:wat::std::stream::spawn-producer
              (:wat::core::lambda ((_tx :rust::crossbeam_channel::Sender<i64>) -> :())
                ()))
            42
            (:wat::core::lambda ((acc :i64) (x :i64) -> :i64)
              (:wat::core::i64::+ acc x))))
    "#;
    assert!(matches!(run(src), Value::i64(42)));
}

// ─── chunks ──────────────────────────────────────────────────────────

#[test]
fn chunks_groups_by_size_flushes_remainder() {
    // 7 items, size 3 → [[1,2,3], [4,5,6], [7]]. The partial final
    // chunk flushes on upstream disconnect — the core pattern for
    // every future stateful-stage with EOS cleanup.
    let src = r#"

        (:wat::core::define (:user::main -> :Vec<Vec<i64>>)
          (:wat::std::stream::collect
            (:wat::std::stream::chunks
              (:wat::std::stream::spawn-producer
                (:wat::core::lambda ((tx :rust::crossbeam_channel::Sender<i64>) -> :())
                  (:wat::core::let*
                    (((_ :Option<()>) (:wat::kernel::send tx 1))
                     ((_ :Option<()>) (:wat::kernel::send tx 2))
                     ((_ :Option<()>) (:wat::kernel::send tx 3))
                     ((_ :Option<()>) (:wat::kernel::send tx 4))
                     ((_ :Option<()>) (:wat::kernel::send tx 5))
                     ((_ :Option<()>) (:wat::kernel::send tx 6))
                     ((_ :Option<()>) (:wat::kernel::send tx 7)))
                    ())))
              3)))
    "#;
    match run(src) {
        Value::Vec(outer) => {
            let got: Vec<Vec<i64>> = outer
                .iter()
                .map(|inner| match inner {
                    Value::Vec(items) => items
                        .iter()
                        .map(|v| match v {
                            Value::i64(n) => *n,
                            other => panic!("inner expected i64; got {:?}", other),
                        })
                        .collect(),
                    other => panic!("outer expected Vec; got {:?}", other),
                })
                .collect();
            assert_eq!(
                got,
                vec![vec![1, 2, 3], vec![4, 5, 6], vec![7]],
                "chunks should emit full chunks followed by the partial flush"
            );
        }
        other => panic!("expected Vec of Vecs; got {:?}", other),
    }
}

#[test]
fn chunks_with_exact_multiple_emits_no_partial_flush() {
    // 6 items, size 3 → [[1,2,3], [4,5,6]]. No partial flush.
    let src = r#"

        (:wat::core::define (:user::main -> :Vec<Vec<i64>>)
          (:wat::std::stream::collect
            (:wat::std::stream::chunks
              (:wat::std::stream::spawn-producer
                (:wat::core::lambda ((tx :rust::crossbeam_channel::Sender<i64>) -> :())
                  (:wat::core::let*
                    (((_ :Option<()>) (:wat::kernel::send tx 1))
                     ((_ :Option<()>) (:wat::kernel::send tx 2))
                     ((_ :Option<()>) (:wat::kernel::send tx 3))
                     ((_ :Option<()>) (:wat::kernel::send tx 4))
                     ((_ :Option<()>) (:wat::kernel::send tx 5))
                     ((_ :Option<()>) (:wat::kernel::send tx 6)))
                    ())))
              3)))
    "#;
    match run(src) {
        Value::Vec(outer) => {
            let got: Vec<Vec<i64>> = outer
                .iter()
                .map(|inner| match inner {
                    Value::Vec(items) => items
                        .iter()
                        .map(|v| match v {
                            Value::i64(n) => *n,
                            other => panic!("{:?}", other),
                        })
                        .collect(),
                    other => panic!("{:?}", other),
                })
                .collect();
            assert_eq!(got, vec![vec![1, 2, 3], vec![4, 5, 6]]);
        }
        other => panic!("{:?}", other),
    }
}

#[test]
fn chunks_into_map_composes() {
    // 5 items, size 2, then map each batch to its sum.
    // [[1,2], [3,4], [5]] → [3, 7, 5].
    let src = r#"

        (:wat::core::define (:user::main -> :Vec<i64>)
          (:wat::std::stream::collect
            (:wat::std::stream::map
              (:wat::std::stream::chunks
                (:wat::std::stream::spawn-producer
                  (:wat::core::lambda ((tx :rust::crossbeam_channel::Sender<i64>) -> :())
                    (:wat::core::let*
                      (((_ :Option<()>) (:wat::kernel::send tx 1))
                       ((_ :Option<()>) (:wat::kernel::send tx 2))
                       ((_ :Option<()>) (:wat::kernel::send tx 3))
                       ((_ :Option<()>) (:wat::kernel::send tx 4))
                       ((_ :Option<()>) (:wat::kernel::send tx 5)))
                      ())))
                2)
              (:wat::core::lambda ((batch :Vec<i64>) -> :i64)
                (:wat::core::foldl batch 0
                  (:wat::core::lambda ((acc :i64) (x :i64) -> :i64)
                    (:wat::core::i64::+ acc x)))))))
    "#;
    assert_eq!(collected_i64(src), vec![3, 7, 5]);
}

// ─── take ────────────────────────────────────────────────────────────

#[test]
fn take_cuts_off_at_n_with_producer_that_would_send_more() {
    // Producer sends 10 items; take 3. The producer would keep
    // going, but bounded(1) blocks it after take's worker exits;
    // the next send returns :None so the producer exits too. This
    // is the core test that take's drop cascade works.
    let src = r#"

        (:wat::core::define (:user::main -> :Vec<i64>)
          (:wat::core::let*
            (((source :wat::std::stream::Stream<i64>)
              (:wat::std::stream::spawn-producer
                (:wat::core::lambda ((tx :rust::crossbeam_channel::Sender<i64>) -> :())
                  (:wat::core::let*
                    (((_ :Option<()>) (:wat::kernel::send tx 1))
                     ((_ :Option<()>) (:wat::kernel::send tx 2))
                     ((_ :Option<()>) (:wat::kernel::send tx 3))
                     ((_ :Option<()>) (:wat::kernel::send tx 4))
                     ((_ :Option<()>) (:wat::kernel::send tx 5))
                     ((_ :Option<()>) (:wat::kernel::send tx 6))
                     ((_ :Option<()>) (:wat::kernel::send tx 7))
                     ((_ :Option<()>) (:wat::kernel::send tx 8))
                     ((_ :Option<()>) (:wat::kernel::send tx 9))
                     ((_ :Option<()>) (:wat::kernel::send tx 10)))
                    ()))))
             ((taken :wat::std::stream::Stream<i64>)
              (:wat::std::stream::take source 3)))
            (:wat::std::stream::collect taken)))
    "#;
    assert_eq!(collected_i64(src), vec![1, 2, 3]);
}

#[test]
fn take_returns_all_when_n_exceeds_available() {
    // Producer has 2 items; take 5. take sees :None before
    // counter hits 0; exits cleanly; collect returns the 2 items.
    let src = r#"

        (:wat::core::define (:user::main -> :Vec<i64>)
          (:wat::core::let*
            (((source :wat::std::stream::Stream<i64>)
              (:wat::std::stream::spawn-producer
                (:wat::core::lambda ((tx :rust::crossbeam_channel::Sender<i64>) -> :())
                  (:wat::core::let*
                    (((_ :Option<()>) (:wat::kernel::send tx 100))
                     ((_ :Option<()>) (:wat::kernel::send tx 200)))
                    ()))))
             ((taken :wat::std::stream::Stream<i64>)
              (:wat::std::stream::take source 5)))
            (:wat::std::stream::collect taken)))
    "#;
    assert_eq!(collected_i64(src), vec![100, 200]);
}

#[test]
fn take_zero_emits_nothing() {
    // take 0 → worker exits immediately; downstream sees :None
    // on first recv; collect returns empty.
    let src = r#"

        (:wat::core::define (:user::main -> :Vec<i64>)
          (:wat::core::let*
            (((source :wat::std::stream::Stream<i64>)
              (:wat::std::stream::spawn-producer
                (:wat::core::lambda ((tx :rust::crossbeam_channel::Sender<i64>) -> :())
                  (:wat::core::let*
                    (((_ :Option<()>) (:wat::kernel::send tx 1))
                     ((_ :Option<()>) (:wat::kernel::send tx 2)))
                    ()))))
             ((taken :wat::std::stream::Stream<i64>)
              (:wat::std::stream::take source 0)))
            (:wat::std::stream::collect taken)))
    "#;
    assert_eq!(collected_i64(src), Vec::<i64>::new());
}

#[test]
fn take_composes_with_map() {
    // source → map(+10) → take(2) → collect. Proves take's
    // drop cascade propagates back through a map stage to the
    // producer.
    let src = r#"

        (:wat::core::define (:user::main -> :Vec<i64>)
          (:wat::core::let*
            (((source :wat::std::stream::Stream<i64>)
              (:wat::std::stream::spawn-producer
                (:wat::core::lambda ((tx :rust::crossbeam_channel::Sender<i64>) -> :())
                  (:wat::core::let*
                    (((_ :Option<()>) (:wat::kernel::send tx 1))
                     ((_ :Option<()>) (:wat::kernel::send tx 2))
                     ((_ :Option<()>) (:wat::kernel::send tx 3))
                     ((_ :Option<()>) (:wat::kernel::send tx 4))
                     ((_ :Option<()>) (:wat::kernel::send tx 5)))
                    ()))))
             ((mapped :wat::std::stream::Stream<i64>)
              (:wat::std::stream::map source
                (:wat::core::lambda ((n :i64) -> :i64)
                  (:wat::core::i64::+ n 10))))
             ((taken :wat::std::stream::Stream<i64>)
              (:wat::std::stream::take mapped 2)))
            (:wat::std::stream::collect taken)))
    "#;
    assert_eq!(collected_i64(src), vec![11, 12]);
}

// ─── inspect ─────────────────────────────────────────────────────────

#[test]
fn inspect_passes_values_through_unchanged() {
    // inspect with a no-op side effect — values must reach collect
    // identical to the source. Validates the pipeline shape even
    // before the effect is observable.
    let src = r#"

        (:wat::core::define (:user::main -> :Vec<i64>)
          (:wat::core::let*
            (((source :wat::std::stream::Stream<i64>)
              (:wat::std::stream::spawn-producer
                (:wat::core::lambda ((tx :rust::crossbeam_channel::Sender<i64>) -> :())
                  (:wat::core::let*
                    (((_ :Option<()>) (:wat::kernel::send tx 10))
                     ((_ :Option<()>) (:wat::kernel::send tx 20))
                     ((_ :Option<()>) (:wat::kernel::send tx 30)))
                    ()))))
             ((inspected :wat::std::stream::Stream<i64>)
              (:wat::std::stream::inspect source
                (:wat::core::lambda ((_n :i64) -> :()) ()))))
            (:wat::std::stream::collect inspected)))
    "#;
    assert_eq!(collected_i64(src), vec![10, 20, 30]);
}

#[test]
fn inspect_composes_between_map_and_collect() {
    // source → map(+1) → inspect(noop) → map(*10) → collect.
    // Four stages; inspect in the middle must be a transparent
    // pass-through — output = (n+1)*10 per input.
    let src = r#"

        (:wat::core::define (:user::main -> :Vec<i64>)
          (:wat::core::let*
            (((s0 :wat::std::stream::Stream<i64>)
              (:wat::std::stream::spawn-producer
                (:wat::core::lambda ((tx :rust::crossbeam_channel::Sender<i64>) -> :())
                  (:wat::core::let*
                    (((_ :Option<()>) (:wat::kernel::send tx 1))
                     ((_ :Option<()>) (:wat::kernel::send tx 2))
                     ((_ :Option<()>) (:wat::kernel::send tx 3)))
                    ()))))
             ((s1 :wat::std::stream::Stream<i64>)
              (:wat::std::stream::map s0
                (:wat::core::lambda ((n :i64) -> :i64)
                  (:wat::core::i64::+ n 1))))
             ((s2 :wat::std::stream::Stream<i64>)
              (:wat::std::stream::inspect s1
                (:wat::core::lambda ((_n :i64) -> :()) ())))
             ((s3 :wat::std::stream::Stream<i64>)
              (:wat::std::stream::map s2
                (:wat::core::lambda ((n :i64) -> :i64)
                  (:wat::core::i64::* n 10)))))
            (:wat::std::stream::collect s3)))
    "#;
    assert_eq!(collected_i64(src), vec![20, 30, 40]);
}

// ─── flat-map ────────────────────────────────────────────────────────

#[test]
fn flat_map_expands_each_input_to_two_outputs() {
    // 1:N — each n becomes [n, n*10]. 3 inputs → 6 outputs.
    let src = r#"

        (:wat::core::define (:user::main -> :Vec<i64>)
          (:wat::core::let*
            (((source :wat::std::stream::Stream<i64>)
              (:wat::std::stream::spawn-producer
                (:wat::core::lambda ((tx :rust::crossbeam_channel::Sender<i64>) -> :())
                  (:wat::core::let*
                    (((_ :Option<()>) (:wat::kernel::send tx 1))
                     ((_ :Option<()>) (:wat::kernel::send tx 2))
                     ((_ :Option<()>) (:wat::kernel::send tx 3)))
                    ()))))
             ((expanded :wat::std::stream::Stream<i64>)
              (:wat::std::stream::flat-map source
                (:wat::core::lambda ((n :i64) -> :Vec<i64>)
                  (:wat::core::vec :i64 n (:wat::core::i64::* n 10))))))
            (:wat::std::stream::collect expanded)))
    "#;
    assert_eq!(collected_i64(src), vec![1, 10, 2, 20, 3, 30]);
}

#[test]
fn flat_map_empty_expansion_emits_nothing() {
    // 1:0 sub-case — each expansion returns an empty Vec; no
    // downstream emissions. collect returns empty Vec.
    let src = r#"

        (:wat::core::define (:user::main -> :Vec<i64>)
          (:wat::core::let*
            (((source :wat::std::stream::Stream<i64>)
              (:wat::std::stream::spawn-producer
                (:wat::core::lambda ((tx :rust::crossbeam_channel::Sender<i64>) -> :())
                  (:wat::core::let*
                    (((_ :Option<()>) (:wat::kernel::send tx 1))
                     ((_ :Option<()>) (:wat::kernel::send tx 2)))
                    ()))))
             ((expanded :wat::std::stream::Stream<i64>)
              (:wat::std::stream::flat-map source
                (:wat::core::lambda ((_n :i64) -> :Vec<i64>)
                  (:wat::core::vec :i64)))))
            (:wat::std::stream::collect expanded)))
    "#;
    assert_eq!(collected_i64(src), Vec::<i64>::new());
}

#[test]
fn flat_map_mixed_expansion_sizes() {
    // Variable expansion — 3 inputs produce [3 items, 0 items, 2 items]
    // → total 5 outputs in input order.
    let src = r#"

        (:wat::core::define (:user::main -> :Vec<i64>)
          (:wat::core::let*
            (((source :wat::std::stream::Stream<i64>)
              (:wat::std::stream::spawn-producer
                (:wat::core::lambda ((tx :rust::crossbeam_channel::Sender<i64>) -> :())
                  (:wat::core::let*
                    (((_ :Option<()>) (:wat::kernel::send tx 1))
                     ((_ :Option<()>) (:wat::kernel::send tx 2))
                     ((_ :Option<()>) (:wat::kernel::send tx 3)))
                    ()))))
             ((expanded :wat::std::stream::Stream<i64>)
              (:wat::std::stream::flat-map source
                (:wat::core::lambda ((n :i64) -> :Vec<i64>)
                  (:wat::core::if (:wat::core::= n 1) -> :Vec<i64>
                    (:wat::core::vec :i64 100 101 102)
                    (:wat::core::if (:wat::core::= n 2) -> :Vec<i64>
                      (:wat::core::vec :i64)
                      (:wat::core::vec :i64 300 301)))))))
            (:wat::std::stream::collect expanded)))
    "#;
    assert_eq!(collected_i64(src), vec![100, 101, 102, 300, 301]);
}
