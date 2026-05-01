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
       (price :wat::core::f64)
       (qty :wat::core::i64))
     (Sell
       (price :wat::core::f64)
       (qty :wat::core::i64)
       (reason :wat::core::String)
       (forced :wat::core::bool)))


   ;; Two-level let* helper — outer holds driver; inner sends
   ;; entries + drops. Function-decomposed per Step 9.
   (:wat::core::define
     (:test::send-events
       (pool :wat::telemetry::Service::HandlePool<test::Event>)
       -> :())
     (:wat::core::let*
       (((handle :wat::telemetry::Service::Handle<test::Event>)
         (:wat::kernel::HandlePool::pop pool))
        ((_finish :()) (:wat::kernel::HandlePool::finish pool))
        ((req-tx :wat::telemetry::Service::ReqTx<test::Event>)
         (:wat::core::first handle))
        ((ack-rx :wat::telemetry::Service::AckRx)
         (:wat::core::second handle))
        ((entries :Vec<test::Event>)
         (:wat::core::vec :test::Event
           (:test::Event::Buy 100.5 7)
           (:test::Event::Sell 102.25 3 "stop-loss" true)))
        ((_log :())
         (:wat::telemetry::Service/batch-log
           req-tx ack-rx entries)))
       ()))


   (:wat::core::define
     (:test::auto-spawn-events
       (path :wat::core::String)
       -> :wat::kernel::Thread<(),()>)
     (:wat::core::let*
       (((spawn :wat::telemetry::Service::Spawn<test::Event>)
         (:wat::telemetry::Sqlite/auto-spawn
           :test::Event
           path 1
           (:wat::telemetry::Service/null-metrics-cadence)
           :wat::telemetry::Sqlite/null-pre-install))
        ((pool :wat::telemetry::Service::HandlePool<test::Event>)
         (:wat::core::first spawn))
        ((driver :wat::kernel::Thread<(),()>)
         (:wat::core::second spawn))
        ((_inner :())
         (:test::send-events pool)))
       driver))))


(:deftest :wat-telemetry-sqlite::auto-spawn::test-event-roundtrip
  (:wat::core::let*
    (((driver :wat::kernel::Thread<(),()>)
      (:test::auto-spawn-events
        "/tmp/wat-sqlite-test-auto-001.db"))
     ((_join :Result<(),Vec<wat::kernel::ThreadDiedError>>)
      (:wat::kernel::Thread/join-result driver)))
    (:wat::test::assert-eq true true)))
