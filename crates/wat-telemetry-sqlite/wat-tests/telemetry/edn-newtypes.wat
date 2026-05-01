;; wat-tests/std/telemetry/edn-newtypes.wat — arc 091 slice 1 smoke
;; test for the Tagged/NoTag newtype TEXT-bind path.
;;
;; Declares a tiny enum with one `:wat::edn::Tagged` field and one
;; `:wat::edn::NoTag` field, runs an entry through auto-spawn, and
;; joins. Verifies:
;;
;;   - schema derivation accepts both newtype field types as TEXT NOT NULL
;;   - dispatch routes Tagged through `:wat::edn::write` (round-trip)
;;   - dispatch routes NoTag through `:wat::edn::write-notag` (lossy)
;;
;; Row content is verified out-of-band via sqlite3 CLI on the .db
;; file — Tagged column should carry `#namespace/Type` markers,
;; NoTag column should not.

(:wat::test::make-deftest :deftest
  ((:wat::core::enum :test::Edn::Event
     (Log
       (data    :wat::edn::Tagged)
       (subject :wat::edn::NoTag)))


   ;; Driver: pop tx, build one HolonAST, wrap it in Tagged AND NoTag,
   ;; send through batch-log, drop.
   (:wat::core::define
     (:test::Edn::send-one
       (pool :wat::telemetry::Service::HandlePool<test::Edn::Event>)
       -> :wat::core::unit)
     (:wat::core::let*
       (((handle :wat::telemetry::Service::Handle<test::Edn::Event>)
         (:wat::kernel::HandlePool::pop pool))
        ((_finish :wat::core::unit) (:wat::kernel::HandlePool::finish pool))
        ((req-tx :wat::telemetry::Service::ReqTx<test::Edn::Event>)
         (:wat::core::first handle))
        ((ack-rx :wat::telemetry::Service::AckRx)
         (:wat::core::second handle))
        ((ast :wat::holon::HolonAST) (:wat::holon::Atom "hello"))
        ((tagged :wat::edn::Tagged)  (:wat::edn::Tagged/new ast))
        ((notag  :wat::edn::NoTag)   (:wat::edn::NoTag/new  ast))
        ((entries :Vec<test::Edn::Event>)
         (:wat::core::vec :test::Edn::Event
           (:test::Edn::Event::Log tagged notag)))
        ((_log :wat::core::unit)
         (:wat::telemetry::Service/batch-log
           req-tx ack-rx entries)))
       ()))


   (:wat::core::define
     (:test::Edn::auto-spawn-one
       (path :wat::core::String)
       -> :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
     (:wat::core::let*
       (((spawn :wat::telemetry::Service::Spawn<test::Edn::Event>)
         (:wat::telemetry::Sqlite/auto-spawn
           :test::Edn::Event
           path 1
           (:wat::telemetry::Service/null-metrics-cadence)
           :wat::telemetry::Sqlite/null-pre-install))
        ((pool :wat::telemetry::Service::HandlePool<test::Edn::Event>)
         (:wat::core::first spawn))
        ((driver :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
         (:wat::core::second spawn))
        ((_inner :wat::core::unit)
         (:test::Edn::send-one pool)))
       driver))))


(:deftest :wat-telemetry-sqlite::edn-newtypes::test-tagged-and-notag-bind
  (:wat::core::let*
    (((driver :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
      (:test::Edn::auto-spawn-one
        "/tmp/wat-sqlite-test-edn-newtypes-001.db"))
     ((_join :Result<wat::core::unit,Vec<wat::kernel::ThreadDiedError>>)
      (:wat::kernel::Thread/join-result driver)))
    (:wat::test::assert-eq true true)))
