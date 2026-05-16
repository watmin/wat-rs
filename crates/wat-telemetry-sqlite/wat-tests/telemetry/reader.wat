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
;;   + Tagged/HolonAST + wat::core::HashMap<HolonAST,HolonAST>)
;; - stream::spawn-producer + stream::collect work end-to-end
;;   over the cursor

(:wat::test::make-deftest :deftest
  (;; Build one Event::Log entry. Mirrors the WorkUnitLog/log
   ;; shape (the writer-side production path) but constructed
   ;; directly so we don't need a WorkUnit for the test.
   (:wat::core::define
     (:test::reader::make-log
       (time-ns :wat::core::i64)
       (msg :wat::core::String)
       -> :wat::telemetry::Event)
     (:wat::core::let
       [ns-ast (:wat::holon::leaf :test::reader)
        cal-ast (:wat::holon::leaf :test::reader::roundtrip)
        lvl-ast (:wat::holon::leaf :info)
        data-ast (:wat::holon::leaf msg)
        ns-notag  (:wat::edn::NoTag/new ns-ast)
        cal-notag  (:wat::edn::NoTag/new cal-ast)
        lvl-notag  (:wat::edn::NoTag/new lvl-ast)
        data-tag (:wat::edn::Tagged/new data-ast)
        tags
         (:wat::core::HashMap
           :(wat::holon::HolonAST,wat::holon::HolonAST))]
       (:wat::telemetry::Event::Log
         time-ns ns-notag cal-notag lvl-notag
         "test-reader-uuid" tags data-tag)))


   (:wat::core::define
     (:test::reader::write-three
       (pool :wat::telemetry::HandlePool<wat::telemetry::Event>)
       -> :wat::core::nil)
     (:wat::core::let
       [handle
         (:wat::kernel::HandlePool::pop pool)
        _finish (:wat::kernel::HandlePool::finish pool)
        req-tx
         (:wat::core::first handle)
        ack-rx
         (:wat::core::second handle)
        entries
         (:wat::core::Vector :wat::telemetry::Event
           (:test::reader::make-log 1000 "first")
           (:test::reader::make-log 2000 "second")
           (:test::reader::make-log 3000 "third"))
        _log
         (:wat::telemetry::batch-log
           req-tx ack-rx entries)]
       ()))


   (:wat::core::define
     (:test::reader::write-fixture
       (path :wat::core::String)
       -> :wat::kernel::Thread<wat::core::nil,wat::core::nil>)
     (:wat::core::let
       [spawn
         (:wat::telemetry::Sqlite/auto-spawn
           :wat::telemetry::Event
           path 1
           (:wat::telemetry::null-metrics-cadence)
           :wat::telemetry::Sqlite/null-pre-install)
        pool
         (:wat::core::first spawn)
        driver
         (:wat::core::second spawn)
        _inner
         (:test::reader::write-three pool)]
       driver))))


;; Round-trip the three Log rows through writer + reader with
;; an empty constraint vec (full-table scan).
;;
;; Arc 132 — explicit override of the default 200ms budget. This
;; test spawns a sqlite-writer thread, opens the .db, streams rows
;; back through a producer thread, and joins both. The combined
;; thread-spawn + sqlite I/O latency exceeds 200ms in practice; 2s
;; is generous headroom for CI noise while still catching genuine
;; deadlocks. Same rationale applies to the five reader tests
;; below.
(:wat::test::time-limit "2s")
(:deftest :wat-telemetry-sqlite::reader::test-roundtrip-three-logs
  (:wat::core::let
    [;; Phase 1 — write fixture. Auto-deleting TempFile so the
     ;; .db unlinks at let scope exit (Drop fires when the
     ;; binding's Arc-count reaches zero); no /tmp leak across
     ;; test runs.
     tf (:wat::io::TempFile/new)
     path (:wat::io::TempFile/path tf)
     driver
      (:test::reader::write-fixture path)
     _join
      (:wat::kernel::Thread/drain-and-join driver)

     ;; Phase 2 — open as ReadHandle and stream the rows back.
     ;; Empty constraint vec = full-table scan.
     handle
      (:wat::sqlite::open-readonly path)
     no-constraints
      (:wat::core::Vector :wat::telemetry::TimeConstraint)
     stream
      (:wat::telemetry::sqlite/stream-logs handle no-constraints)
     events
      (:wat::stream::collect stream)
     count (:wat::core::length events)]
    (:wat::test::assert-eq count 3)))


;; Slice 2 — verify the WHERE pushdown actually narrows. Fixture
;; rows have time_ns ∈ {1000, 2000, 3000}; a Since cutoff at 2000
;; should yield only the {2000, 3000} pair.
(:wat::test::time-limit "2s")
(:deftest :wat-telemetry-sqlite::reader::test-since-narrowing
  (:wat::core::let
    [tf (:wat::io::TempFile/new)
     path (:wat::io::TempFile/path tf)
     driver
      (:test::reader::write-fixture path)
     _join
      (:wat::kernel::Thread/drain-and-join driver)

     handle
      (:wat::sqlite::open-readonly path)
     ;; Since(instant @ time_ns=2000). The fixture writes rows
     ;; with time_ns = 1000, 2000, 3000 — Since 2000 keeps the
     ;; latter two.
     cutoff (:wat::time::at-nanos 2000)
     constraints
      (:wat::core::Vector :wat::telemetry::TimeConstraint
        (:wat::telemetry::since cutoff))
     stream
      (:wat::telemetry::sqlite/stream-logs handle constraints)
     events
      (:wat::stream::collect stream)
     count (:wat::core::length events)]
    (:wat::test::assert-eq count 2)))


;; Slice 2 — Until cutoff drops the newer rows.
(:wat::test::time-limit "2s")
(:deftest :wat-telemetry-sqlite::reader::test-until-narrowing
  (:wat::core::let
    [tf (:wat::io::TempFile/new)
     path (:wat::io::TempFile/path tf)
     driver
      (:test::reader::write-fixture path)
     _join
      (:wat::kernel::Thread/drain-and-join driver)

     handle
      (:wat::sqlite::open-readonly path)
     ;; Until(instant @ time_ns=1500) — only the time_ns=1000 row.
     cutoff (:wat::time::at-nanos 1500)
     constraints
      (:wat::core::Vector :wat::telemetry::TimeConstraint
        (:wat::telemetry::until cutoff))
     stream
      (:wat::telemetry::sqlite/stream-logs handle constraints)
     events
      (:wat::stream::collect stream)
     count (:wat::core::length events)]
    (:wat::test::assert-eq count 1)))


;; Slice 2 — Since AND Until compose to a window.
(:wat::test::time-limit "2s")
(:deftest :wat-telemetry-sqlite::reader::test-since-and-until-window
  (:wat::core::let
    [tf (:wat::io::TempFile/new)
     path (:wat::io::TempFile/path tf)
     driver
      (:test::reader::write-fixture path)
     _join
      (:wat::kernel::Thread/drain-and-join driver)

     handle
      (:wat::sqlite::open-readonly path)
     ;; Since(1500) AND Until(2500) — only the time_ns=2000 row.
     lo (:wat::time::at-nanos 1500)
     hi (:wat::time::at-nanos 2500)
     constraints
      (:wat::core::Vector :wat::telemetry::TimeConstraint
        (:wat::telemetry::since lo)
        (:wat::telemetry::until hi))
     stream
      (:wat::telemetry::sqlite/stream-logs handle constraints)
     events
      (:wat::stream::collect stream)
     count (:wat::core::length events)]
    (:wat::test::assert-eq count 1)))


;; Slice 3 — data-ast extracts the Tagged HolonAST from a Log
;; event. The fixture writes data via `(:leaf msg)` for each
;; row; data-ast unwraps it back to a HolonAST::String leaf;
;; atom-value extracts the original message string.
(:wat::test::time-limit "2s")
(:deftest :wat-telemetry-sqlite::reader::test-data-ast-extracts-holon
  (:wat::core::let
    [tf (:wat::io::TempFile/new)
     path (:wat::io::TempFile/path tf)
     driver
      (:test::reader::write-fixture path)
     _join
      (:wat::kernel::Thread/drain-and-join driver)
     handle
      (:wat::sqlite::open-readonly path)
     no-constraints
      (:wat::core::Vector :wat::telemetry::TimeConstraint)
     events
      (:wat::stream::collect
        (:wat::telemetry::sqlite/stream-logs handle no-constraints))
     ;; First event is the {time_ns=1000, "first"} row.
     first-evt
      (:wat::core::match (:wat::core::first events) -> :wat::telemetry::Event
        ((:wat::core::Some e) e)
        (:wat::core::None
          (:wat::test::assertion-failed
            "expected at least one event")))
     ;; data-ast returns Some(HolonAST::String "first").
     msg
      (:wat::core::match
        (:wat::telemetry::Event::Log/data-ast first-evt)
        -> :wat::core::String
        ((:wat::core::Some h) (:wat::core::atom-value h))
        (:wat::core::None "fail"))]
    (:wat::test::assert-eq msg "first")))


;; Slice 3 — data-value<:wat::core::String> lifts the Tagged AST to a bare
;; String via eval-ast!. Same fixture as data-ast, but skips the
;; explicit atom-value step — the lift goes straight to T.
(:wat::test::time-limit "2s")
(:deftest :wat-telemetry-sqlite::reader::test-data-value-lifts-string
  (:wat::core::let
    [tf (:wat::io::TempFile/new)
     path (:wat::io::TempFile/path tf)
     driver
      (:test::reader::write-fixture path)
     _join
      (:wat::kernel::Thread/drain-and-join driver)
     handle
      (:wat::sqlite::open-readonly path)
     no-constraints
      (:wat::core::Vector :wat::telemetry::TimeConstraint)
     events
      (:wat::stream::collect
        (:wat::telemetry::sqlite/stream-logs handle no-constraints))
     first-evt
      (:wat::core::match (:wat::core::first events) -> :wat::telemetry::Event
        ((:wat::core::Some e) e)
        (:wat::core::None
          (:wat::test::assertion-failed
            "expected at least one event")))
     ;; data-value<:wat::core::String> — lift Tagged HolonAST → String.
     msg
      (:wat::core::match
        (:wat::telemetry::Event::Log/data-value first-evt)
        -> :wat::core::String
        ((:wat::core::Some s) s)
        (:wat::core::None "fail"))]
    (:wat::test::assert-eq msg "first")))
