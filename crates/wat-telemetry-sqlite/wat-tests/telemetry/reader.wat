;; wat-tests/telemetry/reader.wat — arc 093 slice 1e end-to-end.
;;
;; Round-trip: write 3 Event::Log entries via the auto-spawn
;; writer (arc 091/096 path), close, reopen with the new
;; ReadHandle, stream rows back via stream-logs + collect,
;; assert the count.
;;
;; Verifies:
;;
;; - Read-only ReadHandle opens an existing .db file
;; - LogCursor's Rust producer thread iterates rows and ships
;;   them through the bounded(1) channel
;; - Each row reifies to a Value::Enum :wat::telemetry::Event::Log
;;   with all 7 fields decoded (i64 + String + NoTag/HolonAST x3
;;   + Tagged/HolonAST + HashMap<HolonAST,HolonAST>)
;; - stream::spawn-producer + stream::collect work end-to-end
;;   over the cursor

(:wat::test::make-deftest :deftest
  (;; Build one Event::Log entry. Mirrors the WorkUnitLog/log
   ;; shape (the writer-side production path) but constructed
   ;; directly so we don't need a WorkUnit for the test.
   (:wat::core::define
     (:test::reader::make-log
       (time-ns :i64)
       (msg :String)
       -> :wat::telemetry::Event)
     (:wat::core::let*
       (((ns-ast    :wat::holon::HolonAST) (:wat::holon::leaf :test::reader))
        ((cal-ast   :wat::holon::HolonAST) (:wat::holon::leaf :test::reader::roundtrip))
        ((lvl-ast   :wat::holon::HolonAST) (:wat::holon::leaf :info))
        ((data-ast  :wat::holon::HolonAST) (:wat::holon::leaf msg))
        ((ns-notag  :wat::edn::NoTag)  (:wat::edn::NoTag/new ns-ast))
        ((cal-notag :wat::edn::NoTag)  (:wat::edn::NoTag/new cal-ast))
        ((lvl-notag :wat::edn::NoTag)  (:wat::edn::NoTag/new lvl-ast))
        ((data-tag  :wat::edn::Tagged) (:wat::edn::Tagged/new data-ast))
        ((tags :wat::telemetry::Tags)
         (:wat::core::HashMap
           :(wat::holon::HolonAST,wat::holon::HolonAST))))
       (:wat::telemetry::Event::Log
         time-ns ns-notag cal-notag lvl-notag
         "test-reader-uuid" tags data-tag)))


   (:wat::core::define
     (:test::reader::write-three
       (pool :wat::telemetry::Service::HandlePool<wat::telemetry::Event>)
       -> :())
     (:wat::core::let*
       (((handle :wat::telemetry::Service::Handle<wat::telemetry::Event>)
         (:wat::kernel::HandlePool::pop pool))
        ((_finish :()) (:wat::kernel::HandlePool::finish pool))
        ((req-tx :wat::telemetry::Service::ReqTx<wat::telemetry::Event>)
         (:wat::core::first handle))
        ((ack-rx :wat::telemetry::Service::AckRx)
         (:wat::core::second handle))
        ((entries :Vec<wat::telemetry::Event>)
         (:wat::core::vec :wat::telemetry::Event
           (:test::reader::make-log 1000 "first")
           (:test::reader::make-log 2000 "second")
           (:test::reader::make-log 3000 "third")))
        ((_log :())
         (:wat::telemetry::Service/batch-log
           req-tx ack-rx entries)))
       ()))


   (:wat::core::define
     (:test::reader::write-fixture
       (path :String)
       -> :wat::kernel::ProgramHandle<()>)
     (:wat::core::let*
       (((spawn :wat::telemetry::Service::Spawn<wat::telemetry::Event>)
         (:wat::telemetry::Sqlite/auto-spawn
           :wat::telemetry::Event
           path 1
           (:wat::telemetry::Service/null-metrics-cadence)
           :wat::telemetry::Sqlite/null-pre-install))
        ((pool :wat::telemetry::Service::HandlePool<wat::telemetry::Event>)
         (:wat::core::first spawn))
        ((driver :wat::kernel::ProgramHandle<()>)
         (:wat::core::second spawn))
        ((_inner :())
         (:test::reader::write-three pool)))
       driver))))


;; Round-trip the three Log rows through writer + reader.
(:deftest :wat-telemetry-sqlite::reader::test-roundtrip-three-logs
  (:wat::core::let*
    (;; Phase 1 — write fixture. Auto-deleting TempFile so the
     ;; .db unlinks at let* scope exit (Drop fires when the
     ;; binding's Arc-count reaches zero); no /tmp leak across
     ;; test runs.
     ((tf :wat::io::TempFile) (:wat::io::TempFile/new))
     ((path :String) (:wat::io::TempFile/path tf))
     ((driver :wat::kernel::ProgramHandle<()>)
      (:test::reader::write-fixture path))
     ((_join :()) (:wat::kernel::join driver))

     ;; Phase 2 — open as ReadHandle and stream the rows back.
     ((handle :wat::sqlite::ReadHandle)
      (:wat::sqlite::open-readonly path))
     ((query :wat::telemetry::LogQuery)
      (:wat::telemetry::LogQuery/new))
     ((stream :wat::std::stream::Stream<wat::telemetry::Event>)
      (:wat::telemetry::sqlite/stream-logs handle query))
     ((events :Vec<wat::telemetry::Event>)
      (:wat::std::stream::collect stream))
     ((count :i64) (:wat::core::length events)))
    (:wat::test::assert-eq count 3)))
