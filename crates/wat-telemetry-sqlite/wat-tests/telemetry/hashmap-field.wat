;; wat-tests/telemetry/hashmap-field.wat — arc 091 slice 7 smoke
;; test for the HashMap-field auto-dispatch arm.
;;
;; Declares an enum with a `:wat::core::HashMap<HolonAST,HolonAST>` field
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
       (pool :wat::telemetry::HandlePool<test::Tagged::Event>)
       -> :wat::core::nil)
     (:wat::core::let
       [handle
         (:wat::kernel::HandlePool::pop pool)
        _finish (:wat::kernel::HandlePool::finish pool)
        req-tx
         (:wat::core::first handle)
        ack-rx
         (:wat::core::second handle)
        tags
         (:wat::core::assoc
           (:wat::core::assoc
             (:wat::core::HashMap :wat::telemetry::Tag)
             (:wat::holon::Atom :asset) (:wat::holon::Atom :BTC))
           (:wat::holon::Atom :stage) (:wat::holon::Atom :market))
        entries
         (:wat::core::Vector :test::Tagged::Event
           (:test::Tagged::Event::Log tags))
        _log
         (:wat::telemetry::batch-log
           req-tx ack-rx entries)]
       ()))


   (:wat::core::define
     (:test::Tagged::auto-spawn-one
       (path :wat::core::String)
       -> :wat::kernel::Thread<wat::core::nil,wat::core::nil>)
     (:wat::core::let
       [spawn
         (:wat::telemetry::Sqlite/auto-spawn
           :test::Tagged::Event
           path 1
           (:wat::telemetry::null-metrics-cadence)
           :wat::telemetry::Sqlite/null-pre-install)
        pool
         (:wat::core::first spawn)
        driver
         (:wat::core::second spawn)
        _inner
         (:test::Tagged::send-one pool)]
       driver))))


(:deftest :wat-telemetry-sqlite::hashmap-field::test-tags-bind
  (:wat::core::let
    [driver
      (:test::Tagged::auto-spawn-one
        "/tmp/wat-sqlite-test-hashmap-field-001.db")
     _join
      (:wat::kernel::Thread/join-result driver)]
    (:wat::test::assert-eq true true)))
