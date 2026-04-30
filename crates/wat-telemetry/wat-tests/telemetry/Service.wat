;; wat-tests/std/telemetry/Service.wat — arc 080 + arc 089 + arc 095
;; smoke tests for the Service<E,G> shell.
;;
;; The substrate Service shell is generic over E (entry type) and G
;; (cadence gate). Each test below uses a tiny entry type — `:wat::core::i64`
;; for the simplest cases — and a stub dispatcher that pushes
;; received entries onto a channel the test drains afterward.
;;
;; Channel topology (arc 095): each client pops a Handle =
;; (ReqTx, AckRx) from the pool. batch-log takes (req-tx, ack-rx,
;; entries) — two channel ends, no ack-tx-in-request weirdness.
;;
;; The four-step progression:
;;   1. spawn + drop + join (no traffic; lifecycle only)
;;   2. one-batch round-trip (dispatcher sees the entries)
;;   3. cadence fires (translator called → entries dispatched)
;;
;; All tests use null-metrics-cadence except the cadence test.

;; ─── Test 1: spawn + drop + join (no traffic) ────────────────────

(:wat::test::deftest :wat-telemetry::test-spawn-drop-join
  ()
  (:wat::core::let*
    (((stub-pair :wat::kernel::QueuePair<i64>)
      (:wat::kernel::make-bounded-queue :wat::core::i64 16))
     ((stub-tx :wat::kernel::QueueSender<i64>) (:wat::core::first stub-pair))
     ((stub-rx :wat::kernel::QueueReceiver<i64>) (:wat::core::second stub-pair))
     ((dispatcher :fn(Vec<i64>)->())
      (:wat::core::lambda ((entries :Vec<i64>) -> :())
        (:wat::core::foldl entries ()
          (:wat::core::lambda ((_acc :()) (e :wat::core::i64) -> :())
            (:wat::core::match (:wat::kernel::send stub-tx e) -> :()
              ((Some _) ())
              (:None ()))))))
     ((stats-translator :fn(wat::telemetry::Service::Stats)->Vec<i64>)
      (:wat::core::lambda
        ((_s :wat::telemetry::Service::Stats) -> :Vec<i64>)
        (:wat::core::vec :wat::core::i64)))
     ((cadence :wat::telemetry::Service::MetricsCadence<()>)
      (:wat::telemetry::Service/null-metrics-cadence))
     ((spawn :wat::telemetry::Service::Spawn<i64>)
      (:wat::telemetry::Service/spawn 1 cadence dispatcher stats-translator))
     ((pool :wat::telemetry::Service::HandlePool<i64>)
      (:wat::core::first spawn))
     ((driver :wat::kernel::ProgramHandle<()>) (:wat::core::second spawn))
     ;; Inner scope: pop handle, drop without sending.
     ((_inner :())
      (:wat::core::let*
        (((handle :wat::telemetry::Service::Handle<i64>)
          (:wat::kernel::HandlePool::pop pool))
         ((_finish :()) (:wat::kernel::HandlePool::finish pool)))
        ()))
     ((_join :()) (:wat::kernel::join driver)))
    (:wat::test::assert-eq true true)))


;; ─── Test 2: one-batch round-trip ────────────────────────────────
;;
;; Send one batch of 3 entries; drain the stub-rx; assert all three
;; arrived in order.

(:wat::test::deftest :wat-telemetry::test-batch-roundtrip
  ()
  (:wat::core::let*
    (((stub-pair :wat::kernel::QueuePair<i64>)
      (:wat::kernel::make-bounded-queue :wat::core::i64 16))
     ((stub-tx :wat::kernel::QueueSender<i64>) (:wat::core::first stub-pair))
     ((stub-rx :wat::kernel::QueueReceiver<i64>) (:wat::core::second stub-pair))
     ((dispatcher :fn(Vec<i64>)->())
      (:wat::core::lambda ((entries :Vec<i64>) -> :())
        (:wat::core::foldl entries ()
          (:wat::core::lambda ((_acc :()) (e :wat::core::i64) -> :())
            (:wat::core::match (:wat::kernel::send stub-tx e) -> :()
              ((Some _) ())
              (:None ()))))))
     ((stats-translator :fn(wat::telemetry::Service::Stats)->Vec<i64>)
      (:wat::core::lambda
        ((_s :wat::telemetry::Service::Stats) -> :Vec<i64>)
        (:wat::core::vec :wat::core::i64)))
     ((cadence :wat::telemetry::Service::MetricsCadence<()>)
      (:wat::telemetry::Service/null-metrics-cadence))
     ((spawn :wat::telemetry::Service::Spawn<i64>)
      (:wat::telemetry::Service/spawn 1 cadence dispatcher stats-translator))
     ((pool :wat::telemetry::Service::HandlePool<i64>)
      (:wat::core::first spawn))
     ((driver :wat::kernel::ProgramHandle<()>) (:wat::core::second spawn))
     ((_inner :())
      (:wat::core::let*
        (((handle :wat::telemetry::Service::Handle<i64>)
          (:wat::kernel::HandlePool::pop pool))
         ((_finish :()) (:wat::kernel::HandlePool::finish pool))
         ((req-tx :wat::telemetry::Service::ReqTx<i64>)
          (:wat::core::first handle))
         ((ack-rx :wat::telemetry::Service::AckRx)
          (:wat::core::second handle))
         ((entries :Vec<i64>) (:wat::core::vec :wat::core::i64 10 20 30))
         ((_log :())
          (:wat::telemetry::Service/batch-log req-tx ack-rx entries)))
        ()))
     ((_join :()) (:wat::kernel::join driver))
     ;; Drain the stub-rx — three Some values, then None.
     ((r1 :Option<i64>) (:wat::kernel::recv stub-rx))
     ((r2 :Option<i64>) (:wat::kernel::recv stub-rx))
     ((r3 :Option<i64>) (:wat::kernel::recv stub-rx))
     ((v1 :wat::core::i64)
      (:wat::core::match r1 -> :wat::core::i64 ((Some v) v) (:None -1)))
     ((v2 :wat::core::i64)
      (:wat::core::match r2 -> :wat::core::i64 ((Some v) v) (:None -1)))
     ((v3 :wat::core::i64)
      (:wat::core::match r3 -> :wat::core::i64 ((Some v) v) (:None -1)))
     ((u1 :()) (:wat::test::assert-eq v1 10))
     ((u2 :()) (:wat::test::assert-eq v2 20)))
    (:wat::test::assert-eq v3 30)))


;; ─── Test 3: cadence fires → translator called ───────────────────

(:wat::test::deftest :wat-telemetry::test-cadence-fires
  ()
  (:wat::core::let*
    (((stub-pair :wat::kernel::QueuePair<i64>)
      (:wat::kernel::make-bounded-queue :wat::core::i64 16))
     ((stub-tx :wat::kernel::QueueSender<i64>) (:wat::core::first stub-pair))
     ((stub-rx :wat::kernel::QueueReceiver<i64>) (:wat::core::second stub-pair))
     ((dispatcher :fn(Vec<i64>)->())
      (:wat::core::lambda ((entries :Vec<i64>) -> :())
        (:wat::core::foldl entries ()
          (:wat::core::lambda ((_acc :()) (e :wat::core::i64) -> :())
            (:wat::core::match (:wat::kernel::send stub-tx e) -> :()
              ((Some _) ())
              (:None ()))))))
     ((stats-translator :fn(wat::telemetry::Service::Stats)->Vec<i64>)
      (:wat::core::lambda
        ((_s :wat::telemetry::Service::Stats) -> :Vec<i64>)
        (:wat::core::vec :wat::core::i64 -1)))
     ((cadence :wat::telemetry::Service::MetricsCadence<i64>)
      (:wat::telemetry::Service::MetricsCadence/new
        0
        (:wat::core::lambda
          ((g :wat::core::i64) (_s :wat::telemetry::Service::Stats) -> :(i64,bool))
          (:wat::core::tuple 0 true))))
     ((spawn :wat::telemetry::Service::Spawn<i64>)
      (:wat::telemetry::Service/spawn 1 cadence dispatcher stats-translator))
     ((pool :wat::telemetry::Service::HandlePool<i64>)
      (:wat::core::first spawn))
     ((driver :wat::kernel::ProgramHandle<()>) (:wat::core::second spawn))
     ((_inner :())
      (:wat::core::let*
        (((handle :wat::telemetry::Service::Handle<i64>)
          (:wat::kernel::HandlePool::pop pool))
         ((_finish :()) (:wat::kernel::HandlePool::finish pool))
         ((req-tx :wat::telemetry::Service::ReqTx<i64>)
          (:wat::core::first handle))
         ((ack-rx :wat::telemetry::Service::AckRx)
          (:wat::core::second handle))
         ((entries :Vec<i64>) (:wat::core::vec :wat::core::i64 100 200))
         ((_log :())
          (:wat::telemetry::Service/batch-log req-tx ack-rx entries)))
        ()))
     ((_join :()) (:wat::kernel::join driver))
     ((r1 :Option<i64>) (:wat::kernel::recv stub-rx))
     ((r2 :Option<i64>) (:wat::kernel::recv stub-rx))
     ((r3 :Option<i64>) (:wat::kernel::recv stub-rx))
     ((v1 :wat::core::i64) (:wat::core::match r1 -> :wat::core::i64 ((Some v) v) (:None 0)))
     ((v2 :wat::core::i64) (:wat::core::match r2 -> :wat::core::i64 ((Some v) v) (:None 0)))
     ((v3 :wat::core::i64) (:wat::core::match r3 -> :wat::core::i64 ((Some v) v) (:None 0)))
     ((u1 :()) (:wat::test::assert-eq v1 100))
     ((u2 :()) (:wat::test::assert-eq v2 200)))
    (:wat::test::assert-eq v3 -1)))
