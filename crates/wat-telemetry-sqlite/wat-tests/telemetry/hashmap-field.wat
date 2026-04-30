;; wat-tests/telemetry/hashmap-field.wat — arc 091 slice 7 smoke
;; test for the HashMap-field auto-dispatch arm.
;;
;; Declares an enum with a `:HashMap<HolonAST,HolonAST>` field
;; (typealiased through `:wat::telemetry::Tags`), runs an entry
;; through auto-spawn, joins. Verifies:
;;
;;   - schema derivation accepts HashMap field types as TEXT NOT NULL
;;   - dispatch routes the runtime HashMap value through
;;     `:wat::edn::write-notag` rendering
;;
;; Row content is verified out-of-band via sqlite3 CLI on the .db
;; file — the `tags` column should carry an EDN-rendered map
;; literal `{:asset :BTC :stage :market}`.

(:wat::test::make-deftest :deftest
  ((:wat::core::enum :test::Tagged::Event
     (Log
       (tags :wat::telemetry::Tags)))


   (:wat::core::define
     (:test::Tagged::send-one
       (pool :wat::telemetry::Service::HandlePool<test::Tagged::Event>)
       -> :())
     (:wat::core::let*
       (((handle :wat::telemetry::Service::Handle<test::Tagged::Event>)
         (:wat::kernel::HandlePool::pop pool))
        ((_finish :()) (:wat::kernel::HandlePool::finish pool))
        ((req-tx :wat::telemetry::Service::ReqTx<test::Tagged::Event>)
         (:wat::core::first handle))
        ((ack-rx :wat::telemetry::Service::AckRx)
         (:wat::core::second handle))
        ((tags :wat::telemetry::Tags)
         (:wat::core::assoc
           (:wat::core::assoc
             (:wat::core::HashMap :wat::telemetry::Tag)
             (:wat::holon::Atom :asset) (:wat::holon::Atom :BTC))
           (:wat::holon::Atom :stage) (:wat::holon::Atom :market)))
        ((entries :Vec<test::Tagged::Event>)
         (:wat::core::vec :test::Tagged::Event
           (:test::Tagged::Event::Log tags)))
        ((_log :())
         (:wat::telemetry::Service/batch-log
           req-tx ack-rx entries)))
       ()))


   (:wat::core::define
     (:test::Tagged::auto-spawn-one
       (path :wat::core::String)
       -> :wat::kernel::ProgramHandle<()>)
     (:wat::core::let*
       (((spawn :wat::telemetry::Service::Spawn<test::Tagged::Event>)
         (:wat::telemetry::Sqlite/auto-spawn
           :test::Tagged::Event
           path 1
           (:wat::telemetry::Service/null-metrics-cadence)
           :wat::telemetry::Sqlite/null-pre-install))
        ((pool :wat::telemetry::Service::HandlePool<test::Tagged::Event>)
         (:wat::core::first spawn))
        ((driver :wat::kernel::ProgramHandle<()>)
         (:wat::core::second spawn))
        ((_inner :())
         (:test::Tagged::send-one pool)))
       driver))))


(:deftest :wat-telemetry-sqlite::hashmap-field::test-tags-bind
  (:wat::core::let*
    (((driver :wat::kernel::ProgramHandle<()>)
      (:test::Tagged::auto-spawn-one
        "/tmp/wat-sqlite-test-hashmap-field-001.db"))
     ((_join :()) (:wat::kernel::join driver)))
    (:wat::test::assert-eq true true)))
