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
    ;; Outer holds only the Thread; inner owns every QueueSender clone
    ;; (stub-pair, stub-tx). When inner returns the Thread, those clones
    ;; drop, the worker sees EOF, join unblocks. SERVICE-PROGRAMS.md §
    ;; "The lockstep" + arc 117.
    (((driver :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
      (:wat::core::let*
        (((stub-pair :wat::kernel::QueuePair<wat::core::i64>)
          (:wat::kernel::make-bounded-queue :wat::core::i64 16))
         ((stub-tx :wat::kernel::QueueSender<wat::core::i64>) (:wat::core::first stub-pair))
         ((stub-rx :wat::kernel::QueueReceiver<wat::core::i64>) (:wat::core::second stub-pair))
         ((dispatcher :fn(wat::core::Vector<wat::core::i64>)->wat::core::unit)
          (:wat::core::lambda ((entries :wat::core::Vector<wat::core::i64>) -> :wat::core::unit)
            (:wat::core::foldl entries ()
              (:wat::core::lambda ((_acc :wat::core::unit) (e :wat::core::i64) -> :wat::core::unit)
                (:wat::core::match (:wat::kernel::send stub-tx e) -> :wat::core::unit
                  ((:wat::core::Ok _) ())
                  ((:wat::core::Err _) ()))))))
         ((stats-translator :fn(wat::telemetry::Stats)->wat::core::Vector<wat::core::i64>)
          (:wat::core::lambda
            ((_s :wat::telemetry::Stats) -> :wat::core::Vector<wat::core::i64>)
            (:wat::core::Vector :wat::core::i64)))
         ((cadence :wat::telemetry::MetricsCadence<wat::core::unit>)
          (:wat::telemetry::null-metrics-cadence))
         ((spawn :wat::telemetry::Spawn<wat::core::i64>)
          (:wat::telemetry::spawn 1 cadence dispatcher stats-translator))
         ((pool :wat::telemetry::HandlePool<wat::core::i64>)
          (:wat::core::first spawn))
         ((d :wat::kernel::Thread<wat::core::unit,wat::core::unit>) (:wat::core::second spawn))
         ;; Inner-inner: pop handle, drop without sending.
         ((_inner :wat::core::unit)
          (:wat::core::let*
            (((handle :wat::telemetry::Handle<wat::core::i64>)
              (:wat::kernel::HandlePool::pop pool))
             ((_finish :wat::core::unit) (:wat::kernel::HandlePool::finish pool)))
            ())))
        d))
     ((_join :wat::core::Result<wat::core::unit,wat::core::Vector<wat::kernel::ThreadDiedError>>)
      (:wat::kernel::Thread/join-result driver)))
    (:wat::test::assert-eq true true)))


;; ─── Test 2: one-batch round-trip ────────────────────────────────
;;
;; Send one batch of 3 entries; drain the stub-rx; assert all three
;; arrived in order.

(:wat::test::deftest :wat-telemetry::test-batch-roundtrip
  ()
  (:wat::core::let*
    ;; Inner owns every QueueSender clone (stub-pair, stub-tx) AND does
    ;; the batch-log work; returns (driver, stub-rx) so outer can join
    ;; AND drain the receiver after the worker exits. SERVICE-PROGRAMS.md
    ;; § "The lockstep" + arc 117.
    (((thr-and-rx :(wat::kernel::Thread<wat::core::unit,wat::core::unit>,wat::kernel::QueueReceiver<wat::core::i64>))
      (:wat::core::let*
        (((stub-pair :wat::kernel::QueuePair<wat::core::i64>)
          (:wat::kernel::make-bounded-queue :wat::core::i64 16))
         ((stub-tx :wat::kernel::QueueSender<wat::core::i64>) (:wat::core::first stub-pair))
         ((stub-rx :wat::kernel::QueueReceiver<wat::core::i64>) (:wat::core::second stub-pair))
         ((dispatcher :fn(wat::core::Vector<wat::core::i64>)->wat::core::unit)
          (:wat::core::lambda ((entries :wat::core::Vector<wat::core::i64>) -> :wat::core::unit)
            (:wat::core::foldl entries ()
              (:wat::core::lambda ((_acc :wat::core::unit) (e :wat::core::i64) -> :wat::core::unit)
                (:wat::core::match (:wat::kernel::send stub-tx e) -> :wat::core::unit
                  ((:wat::core::Ok _) ())
                  ((:wat::core::Err _) ()))))))
         ((stats-translator :fn(wat::telemetry::Stats)->wat::core::Vector<wat::core::i64>)
          (:wat::core::lambda
            ((_s :wat::telemetry::Stats) -> :wat::core::Vector<wat::core::i64>)
            (:wat::core::Vector :wat::core::i64)))
         ((cadence :wat::telemetry::MetricsCadence<wat::core::unit>)
          (:wat::telemetry::null-metrics-cadence))
         ((spawn :wat::telemetry::Spawn<wat::core::i64>)
          (:wat::telemetry::spawn 1 cadence dispatcher stats-translator))
         ((pool :wat::telemetry::HandlePool<wat::core::i64>)
          (:wat::core::first spawn))
         ((d :wat::kernel::Thread<wat::core::unit,wat::core::unit>) (:wat::core::second spawn))
         ((_inner :wat::core::unit)
          (:wat::core::let*
            (((handle :wat::telemetry::Handle<wat::core::i64>)
              (:wat::kernel::HandlePool::pop pool))
             ((_finish :wat::core::unit) (:wat::kernel::HandlePool::finish pool))
             ((req-tx :wat::telemetry::ReqTx<wat::core::i64>)
              (:wat::core::first handle))
             ((ack-rx :wat::telemetry::AckRx)
              (:wat::core::second handle))
             ((entries :wat::core::Vector<wat::core::i64>) (:wat::core::Vector :wat::core::i64 10 20 30))
             ((_log :wat::core::unit)
              (:wat::telemetry::batch-log req-tx ack-rx entries)))
            ())))
        (:wat::core::Tuple d stub-rx)))
     ((driver :wat::kernel::Thread<wat::core::unit,wat::core::unit>) (:wat::core::first thr-and-rx))
     ((stub-rx :wat::kernel::QueueReceiver<wat::core::i64>) (:wat::core::second thr-and-rx))
     ((_join :wat::core::Result<wat::core::unit,wat::core::Vector<wat::kernel::ThreadDiedError>>)
      (:wat::kernel::Thread/join-result driver))
     ;; Drain the stub-rx — three Some values. Match-at-source per arc 110.
     ((v1 :wat::core::i64)
      (:wat::core::match (:wat::kernel::recv stub-rx) -> :wat::core::i64 ((:wat::core::Ok (:wat::core::Some v)) v) ((:wat::core::Ok :wat::core::None) -1) ((:wat::core::Err _) -1)))
     ((v2 :wat::core::i64)
      (:wat::core::match (:wat::kernel::recv stub-rx) -> :wat::core::i64 ((:wat::core::Ok (:wat::core::Some v)) v) ((:wat::core::Ok :wat::core::None) -1) ((:wat::core::Err _) -1)))
     ((v3 :wat::core::i64)
      (:wat::core::match (:wat::kernel::recv stub-rx) -> :wat::core::i64 ((:wat::core::Ok (:wat::core::Some v)) v) ((:wat::core::Ok :wat::core::None) -1) ((:wat::core::Err _) -1)))
     ((u1 :wat::core::unit) (:wat::test::assert-eq v1 10))
     ((u2 :wat::core::unit) (:wat::test::assert-eq v2 20)))
    (:wat::test::assert-eq v3 30)))


;; ─── Test 3: cadence fires → translator called ───────────────────

(:wat::test::deftest :wat-telemetry::test-cadence-fires
  ()
  (:wat::core::let*
    ;; Inner owns every QueueSender clone (stub-pair, stub-tx) AND does
    ;; the batch-log work; returns (driver, stub-rx) so outer can join
    ;; AND drain. SERVICE-PROGRAMS.md § "The lockstep" + arc 117.
    (((thr-and-rx :(wat::kernel::Thread<wat::core::unit,wat::core::unit>,wat::kernel::QueueReceiver<wat::core::i64>))
      (:wat::core::let*
        (((stub-pair :wat::kernel::QueuePair<wat::core::i64>)
          (:wat::kernel::make-bounded-queue :wat::core::i64 16))
         ((stub-tx :wat::kernel::QueueSender<wat::core::i64>) (:wat::core::first stub-pair))
         ((stub-rx :wat::kernel::QueueReceiver<wat::core::i64>) (:wat::core::second stub-pair))
         ((dispatcher :fn(wat::core::Vector<wat::core::i64>)->wat::core::unit)
          (:wat::core::lambda ((entries :wat::core::Vector<wat::core::i64>) -> :wat::core::unit)
            (:wat::core::foldl entries ()
              (:wat::core::lambda ((_acc :wat::core::unit) (e :wat::core::i64) -> :wat::core::unit)
                (:wat::core::match (:wat::kernel::send stub-tx e) -> :wat::core::unit
                  ((:wat::core::Ok _) ())
                  ((:wat::core::Err _) ()))))))
         ((stats-translator :fn(wat::telemetry::Stats)->wat::core::Vector<wat::core::i64>)
          (:wat::core::lambda
            ((_s :wat::telemetry::Stats) -> :wat::core::Vector<wat::core::i64>)
            (:wat::core::Vector :wat::core::i64 -1)))
         ((cadence :wat::telemetry::MetricsCadence<wat::core::i64>)
          (:wat::telemetry::MetricsCadence/new
            0
            (:wat::core::lambda
              ((g :wat::core::i64) (_s :wat::telemetry::Stats) -> :(wat::core::i64,wat::core::bool))
              (:wat::core::Tuple 0 true))))
         ((spawn :wat::telemetry::Spawn<wat::core::i64>)
          (:wat::telemetry::spawn 1 cadence dispatcher stats-translator))
         ((pool :wat::telemetry::HandlePool<wat::core::i64>)
          (:wat::core::first spawn))
         ((d :wat::kernel::Thread<wat::core::unit,wat::core::unit>) (:wat::core::second spawn))
         ((_inner :wat::core::unit)
          (:wat::core::let*
            (((handle :wat::telemetry::Handle<wat::core::i64>)
              (:wat::kernel::HandlePool::pop pool))
             ((_finish :wat::core::unit) (:wat::kernel::HandlePool::finish pool))
             ((req-tx :wat::telemetry::ReqTx<wat::core::i64>)
              (:wat::core::first handle))
             ((ack-rx :wat::telemetry::AckRx)
              (:wat::core::second handle))
             ((entries :wat::core::Vector<wat::core::i64>) (:wat::core::Vector :wat::core::i64 100 200))
             ((_log :wat::core::unit)
              (:wat::telemetry::batch-log req-tx ack-rx entries)))
            ())))
        (:wat::core::Tuple d stub-rx)))
     ((driver :wat::kernel::Thread<wat::core::unit,wat::core::unit>) (:wat::core::first thr-and-rx))
     ((stub-rx :wat::kernel::QueueReceiver<wat::core::i64>) (:wat::core::second thr-and-rx))
     ((_join :wat::core::Result<wat::core::unit,wat::core::Vector<wat::kernel::ThreadDiedError>>)
      (:wat::kernel::Thread/join-result driver))
     ((v1 :wat::core::i64) (:wat::core::match (:wat::kernel::recv stub-rx) -> :wat::core::i64 ((:wat::core::Ok (:wat::core::Some v)) v) ((:wat::core::Ok :wat::core::None) 0) ((:wat::core::Err _) 0)))
     ((v2 :wat::core::i64) (:wat::core::match (:wat::kernel::recv stub-rx) -> :wat::core::i64 ((:wat::core::Ok (:wat::core::Some v)) v) ((:wat::core::Ok :wat::core::None) 0) ((:wat::core::Err _) 0)))
     ((v3 :wat::core::i64) (:wat::core::match (:wat::kernel::recv stub-rx) -> :wat::core::i64 ((:wat::core::Ok (:wat::core::Some v)) v) ((:wat::core::Ok :wat::core::None) 0) ((:wat::core::Err _) 0)))
     ((u1 :wat::core::unit) (:wat::test::assert-eq v1 100))
     ((u2 :wat::core::unit) (:wat::test::assert-eq v2 200)))
    (:wat::test::assert-eq v3 -1)))
