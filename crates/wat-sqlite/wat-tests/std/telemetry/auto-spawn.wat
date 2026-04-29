;; wat-tests/std/telemetry/auto-spawn.wat — arc 085 smoke test.
;;
;; Declares a tiny throwaway enum, spawns the substrate's auto-
;; derived sqlite sink, sends one entry of each variant, drops,
;; joins. Verifies:
;;
;;   - schema derivation runs without error (auto-prep + install)
;;   - dispatch routes Tagged variants to their derived INSERT
;;   - all four scalar field types (i64, f64, String, bool) bind
;;     correctly through the auto-derived path
;;
;; Row counts verified out-of-band via sqlite3 CLI per the rest of
;; this crate's test pattern.

(:wat::test::make-deftest :deftest
  (;; Tiny test enum with two Tagged variants and mixed types.
   (:wat::core::enum :test::Event
     (Buy
       (price :f64)
       (qty :i64))
     (Sell
       (price :f64)
       (qty :i64)
       (reason :String)
       (forced :bool)))


   ;; Two-level let* helper — outer holds driver; inner sends
   ;; entries + drops. Function-decomposed per Step 9.
   (:wat::core::define
     (:test::send-events
       (pool :wat::std::telemetry::Service::HandlePool<test::Event>)
       -> :())
     (:wat::core::let*
       (((handle :wat::std::telemetry::Service::Handle<test::Event>)
         (:wat::kernel::HandlePool::pop pool))
        ((_finish :()) (:wat::kernel::HandlePool::finish pool))
        ((req-tx :wat::std::telemetry::Service::ReqTx<test::Event>)
         (:wat::core::first handle))
        ((ack-rx :wat::std::telemetry::Service::AckRx)
         (:wat::core::second handle))
        ((entries :Vec<test::Event>)
         (:wat::core::vec :test::Event
           (:test::Event::Buy 100.5 7)
           (:test::Event::Sell 102.25 3 "stop-loss" true)))
        ((_log :())
         (:wat::std::telemetry::Service/batch-log
           req-tx ack-rx entries)))
       ()))


   (:wat::core::define
     (:test::auto-spawn-events
       (path :String)
       -> :wat::kernel::ProgramHandle<()>)
     (:wat::core::let*
       (((spawn :wat::std::telemetry::Service::Spawn<test::Event>)
         (:wat::std::telemetry::Sqlite/auto-spawn
           :test::Event
           path 1
           (:wat::std::telemetry::Service/null-metrics-cadence)
           :wat::std::telemetry::Sqlite/null-pre-install))
        ((pool :wat::std::telemetry::Service::HandlePool<test::Event>)
         (:wat::core::first spawn))
        ((driver :wat::kernel::ProgramHandle<()>)
         (:wat::core::second spawn))
        ((_inner :())
         (:test::send-events pool)))
       driver))))


(:deftest :wat-tests::std::telemetry::auto-spawn::test-event-roundtrip
  (:wat::core::let*
    (((driver :wat::kernel::ProgramHandle<()>)
      (:test::auto-spawn-events
        "/tmp/wat-sqlite-test-auto-001.db"))
     ((_join :()) (:wat::kernel::join driver)))
    (:wat::test::assert-eq true true)))
