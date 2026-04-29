;; wat-tests/std/telemetry/Service.wat — arc 080 smoke tests.
;;
;; The substrate Service shell is generic over E (entry type) and G
;; (cadence gate). Each test below uses a tiny entry type — `:i64`
;; for the simplest cases — and a stub dispatcher that pushes
;; received entries onto a channel the test drains afterward.
;;
;; The four-step progression:
;;   1. spawn + drop + join (no traffic; lifecycle only)
;;   2. one-batch round-trip (dispatcher sees the entries)
;;   3. multi-batch (dispatcher sees them in order; ack semantics)
;;   4. cadence fires (translator called → entries dispatched)
;;
;; All tests use null-metrics-cadence except the cadence test.

;; ─── Test 1: spawn + drop + join (no traffic) ────────────────────

(:wat::test::deftest :wat-tests::std::telemetry::test-spawn-drop-join
  ()
  (:wat::core::let*
    (((stub-pair :wat::kernel::QueuePair<i64>)
      (:wat::kernel::make-bounded-queue :i64 16))
     ((stub-tx :wat::kernel::QueueSender<i64>) (:wat::core::first stub-pair))
     ((stub-rx :wat::kernel::QueueReceiver<i64>) (:wat::core::second stub-pair))
     ;; Dispatcher: closure-over stub-tx; sends each entry through.
     ((dispatcher :fn(i64)->())
      (:wat::core::lambda ((e :i64) -> :())
        (:wat::core::match (:wat::kernel::send stub-tx e) -> :()
          ((Some _) ())
          (:None ()))))
     ;; Stats translator: returns empty vec (no self-heartbeat
     ;; entries — null cadence won't fire anyway).
     ((stats-translator :fn(wat::std::telemetry::Service::Stats)->Vec<i64>)
      (:wat::core::lambda
        ((_s :wat::std::telemetry::Service::Stats) -> :Vec<i64>)
        (:wat::core::vec :i64)))
     ((cadence :wat::std::telemetry::Service::MetricsCadence<()>)
      (:wat::std::telemetry::Service/null-metrics-cadence))
     ((spawn :wat::std::telemetry::Service::Spawn<i64>)
      (:wat::std::telemetry::Service/spawn 1 cadence dispatcher stats-translator))
     ((pool :wat::std::telemetry::Service::ReqTxPool<i64>)
      (:wat::core::first spawn))
     ((driver :wat::kernel::ProgramHandle<()>) (:wat::core::second spawn))
     ;; Inner scope: pop handle, drop without sending.
     ((_inner :())
      (:wat::core::let*
        (((tx :wat::std::telemetry::Service::ReqTx<i64>)
          (:wat::kernel::HandlePool::pop pool))
         ((_finish :()) (:wat::kernel::HandlePool::finish pool)))
        ()))
     ((_join :()) (:wat::kernel::join driver)))
    (:wat::test::assert-eq true true)))


;; ─── Test 2: one-batch round-trip ────────────────────────────────
;;
;; Send one batch of 3 entries; drain the stub-rx; assert all three
;; arrived in order.

(:wat::test::deftest :wat-tests::std::telemetry::test-batch-roundtrip
  ()
  (:wat::core::let*
    (((stub-pair :wat::kernel::QueuePair<i64>)
      (:wat::kernel::make-bounded-queue :i64 16))
     ((stub-tx :wat::kernel::QueueSender<i64>) (:wat::core::first stub-pair))
     ((stub-rx :wat::kernel::QueueReceiver<i64>) (:wat::core::second stub-pair))
     ((dispatcher :fn(i64)->())
      (:wat::core::lambda ((e :i64) -> :())
        (:wat::core::match (:wat::kernel::send stub-tx e) -> :()
          ((Some _) ())
          (:None ()))))
     ((stats-translator :fn(wat::std::telemetry::Service::Stats)->Vec<i64>)
      (:wat::core::lambda
        ((_s :wat::std::telemetry::Service::Stats) -> :Vec<i64>)
        (:wat::core::vec :i64)))
     ((cadence :wat::std::telemetry::Service::MetricsCadence<()>)
      (:wat::std::telemetry::Service/null-metrics-cadence))
     ((spawn :wat::std::telemetry::Service::Spawn<i64>)
      (:wat::std::telemetry::Service/spawn 1 cadence dispatcher stats-translator))
     ((pool :wat::std::telemetry::Service::ReqTxPool<i64>)
      (:wat::core::first spawn))
     ((driver :wat::kernel::ProgramHandle<()>) (:wat::core::second spawn))
     ((_inner :())
      (:wat::core::let*
        (((tx :wat::std::telemetry::Service::ReqTx<i64>)
          (:wat::kernel::HandlePool::pop pool))
         ((_finish :()) (:wat::kernel::HandlePool::finish pool))
         ((ack-channel :wat::std::telemetry::Service::AckChannel)
          (:wat::kernel::make-bounded-queue :() 1))
         ((ack-tx :wat::std::telemetry::Service::AckTx)
          (:wat::core::first ack-channel))
         ((ack-rx :wat::std::telemetry::Service::AckRx)
          (:wat::core::second ack-channel))
         ((entries :Vec<i64>) (:wat::core::vec :i64 10 20 30))
         ((_log :())
          (:wat::std::telemetry::Service/batch-log tx ack-tx ack-rx entries)))
        ()))
     ((_join :()) (:wat::kernel::join driver))
     ;; Drain the stub-rx — three Some values, then None.
     ((r1 :Option<i64>) (:wat::kernel::recv stub-rx))
     ((r2 :Option<i64>) (:wat::kernel::recv stub-rx))
     ((r3 :Option<i64>) (:wat::kernel::recv stub-rx))
     ((v1 :i64)
      (:wat::core::match r1 -> :i64 ((Some v) v) (:None -1)))
     ((v2 :i64)
      (:wat::core::match r2 -> :i64 ((Some v) v) (:None -1)))
     ((v3 :i64)
      (:wat::core::match r3 -> :i64 ((Some v) v) (:None -1)))
     ((u1 :()) (:wat::test::assert-eq v1 10))
     ((u2 :()) (:wat::test::assert-eq v2 20)))
    (:wat::test::assert-eq v3 30)))


;; ─── Test 3: cadence fires → translator called ───────────────────
;;
;; Counter-based MetricsCadence<i64> fires every batch (n>=0 fires).
;; After one batch: tick fires; translator returns [-1] sentinel;
;; dispatcher sees the original batch entries PLUS the -1.
;;
;; Note: tick fires AFTER the batch is dispatched and acked, so the
;; ordering is: batch entries first, THEN heartbeat entries.

(:wat::test::deftest :wat-tests::std::telemetry::test-cadence-fires
  ()
  (:wat::core::let*
    (((stub-pair :wat::kernel::QueuePair<i64>)
      (:wat::kernel::make-bounded-queue :i64 16))
     ((stub-tx :wat::kernel::QueueSender<i64>) (:wat::core::first stub-pair))
     ((stub-rx :wat::kernel::QueueReceiver<i64>) (:wat::core::second stub-pair))
     ((dispatcher :fn(i64)->())
      (:wat::core::lambda ((e :i64) -> :())
        (:wat::core::match (:wat::kernel::send stub-tx e) -> :()
          ((Some _) ())
          (:None ()))))
     ;; Translator returns a one-element vec [-1] — sentinel marker.
     ((stats-translator :fn(wat::std::telemetry::Service::Stats)->Vec<i64>)
      (:wat::core::lambda
        ((_s :wat::std::telemetry::Service::Stats) -> :Vec<i64>)
        (:wat::core::vec :i64 -1)))
     ;; Counter cadence — fires every batch (gate >= 0 always).
     ((cadence :wat::std::telemetry::Service::MetricsCadence<i64>)
      (:wat::std::telemetry::Service::MetricsCadence/new
        0
        (:wat::core::lambda
          ((g :i64) (_s :wat::std::telemetry::Service::Stats) -> :(i64,bool))
          (:wat::core::tuple 0 true))))
     ((spawn :wat::std::telemetry::Service::Spawn<i64>)
      (:wat::std::telemetry::Service/spawn 1 cadence dispatcher stats-translator))
     ((pool :wat::std::telemetry::Service::ReqTxPool<i64>)
      (:wat::core::first spawn))
     ((driver :wat::kernel::ProgramHandle<()>) (:wat::core::second spawn))
     ((_inner :())
      (:wat::core::let*
        (((tx :wat::std::telemetry::Service::ReqTx<i64>)
          (:wat::kernel::HandlePool::pop pool))
         ((_finish :()) (:wat::kernel::HandlePool::finish pool))
         ((ack-channel :wat::std::telemetry::Service::AckChannel)
          (:wat::kernel::make-bounded-queue :() 1))
         ((ack-tx :wat::std::telemetry::Service::AckTx)
          (:wat::core::first ack-channel))
         ((ack-rx :wat::std::telemetry::Service::AckRx)
          (:wat::core::second ack-channel))
         ;; One batch of 2 entries. Cadence fires after; sentinel -1
         ;; lands on stub-rx as the third value.
         ((entries :Vec<i64>) (:wat::core::vec :i64 100 200))
         ((_log :())
          (:wat::std::telemetry::Service/batch-log tx ack-tx ack-rx entries)))
        ()))
     ((_join :()) (:wat::kernel::join driver))
     ;; Drain — expect 100, 200, then -1 (sentinel from translator).
     ((r1 :Option<i64>) (:wat::kernel::recv stub-rx))
     ((r2 :Option<i64>) (:wat::kernel::recv stub-rx))
     ((r3 :Option<i64>) (:wat::kernel::recv stub-rx))
     ((v1 :i64) (:wat::core::match r1 -> :i64 ((Some v) v) (:None 0)))
     ((v2 :i64) (:wat::core::match r2 -> :i64 ((Some v) v) (:None 0)))
     ((v3 :i64) (:wat::core::match r3 -> :i64 ((Some v) v) (:None 0)))
     ((u1 :()) (:wat::test::assert-eq v1 100))
     ((u2 :()) (:wat::test::assert-eq v2 200)))
    (:wat::test::assert-eq v3 -1)))
