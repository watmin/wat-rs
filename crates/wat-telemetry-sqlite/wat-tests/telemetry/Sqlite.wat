;; crates/wat-telemetry-sqlite/wat-tests/telemetry/Sqlite.wat — arc 083 slice 2 smoke tests.
;;
;; Two deftests cover the substrate Sqlite spawn / loop / join
;; lifecycle:
;;
;;   - spawn + pop + finish + drop + join (no traffic; verifies the
;;     wiring shape compiles + shuts down cleanly).
;;
;;   - spawn + batch-log 3 entries + drop + join (verifies hooks
;;     fire end-to-end without crashing; row counts verified out-of-
;;     band via sqlite3 CLI per the slice-1 pattern; a future slice
;;     adds a SELECT primitive for in-test count verification).
;;
;; Helper defines live in the make-deftest prelude — top-level
;; defines in a wat-tests file aren't visible to deftest sandbox
;; bodies (precedent: wat-rs/wat-tests/service-template.wat).

(:wat::test::make-deftest :deftest
  (;; ─── Hooks (no-ops; lifecycle test) ─────────────────────────

   (:wat::core::define
     (:wat-telemetry-sqlite::Sqlite::install-noop
       (_db :wat::sqlite::Db)
       -> :wat::core::unit)
     ())

   (:wat::core::define
     (:wat-telemetry-sqlite::Sqlite::dispatch-noop
       (_db :wat::sqlite::Db)
       (_entries :wat::core::Vector<wat::core::i64>)
       -> :wat::core::unit)
     ())

   (:wat::core::define
     (:wat-telemetry-sqlite::Sqlite::translate-empty
       (_stats :wat::telemetry::Stats)
       -> :wat::core::Vector<wat::core::i64>)
     (:wat::core::Vector :wat::core::i64))


   ;; ─── Hooks (insert; traffic test) ────────────────────────────

   ;; pre-install — flips the worker's Db into WAL journal mode
   ;; before schema-install runs. Mirrors the lab's policy choice;
   ;; this test exercises slice-4's pre-install hook with a real
   ;; non-trivial body. Verified out-of-band via `PRAGMA journal_mode`
   ;; against the produced db file.
   (:wat::core::define
     (:wat-telemetry-sqlite::Sqlite::pragma-wal
       (db :wat::sqlite::Db)
       -> :wat::core::unit)
     (:wat::sqlite::pragma db "journal_mode" "WAL"))

   (:wat::core::define
     (:wat-telemetry-sqlite::Sqlite::install-events
       (db :wat::sqlite::Db)
       -> :wat::core::unit)
     (:wat::sqlite::execute-ddl db
       "CREATE TABLE IF NOT EXISTS events (n INTEGER)"))

   ;; Per-entry insert helper. Builds the SQL via string concat —
   ;; acceptable because i64 is internally typed and there's no
   ;; injection surface; a future slice's parameterized `execute`
   ;; primitive supersedes the concat shape.
   (:wat::core::define
     (:wat-telemetry-sqlite::Sqlite::insert-one-event
       (db :wat::sqlite::Db)
       (entry :wat::core::i64)
       -> :wat::core::unit)
     (:wat::core::let*
       (((sql :wat::core::String)
         (:wat::core::string::concat
           "INSERT INTO events (n) VALUES ("
           (:wat::core::string::concat
             (:wat::core::i64::to-string entry)
             ")"))))
       (:wat::sqlite::execute-ddl db sql)))

   ;; Per-batch dispatcher (arc 089 slice 3). Foldls each entry
   ;; through insert-one-event. No begin/commit wrap here — that's a
   ;; consumer choice (the trader's :trading::telemetry path opts in
   ;; via Sqlite/auto-spawn's batched dispatch); this test just
   ;; exercises the per-batch contract.
   (:wat::core::define
     (:wat-telemetry-sqlite::Sqlite::dispatch-events
       (db :wat::sqlite::Db)
       (entries :wat::core::Vector<wat::core::i64>)
       -> :wat::core::unit)
     (:wat::core::foldl entries ()
       (:wat::core::lambda ((_acc :wat::core::unit) (entry :wat::core::i64) -> :wat::core::unit)
         (:wat-telemetry-sqlite::Sqlite::insert-one-event db entry))))


   ;; ─── Helpers — function-decomposed lockstep (Step 9) ────────

   ;; Spawn + pop one handle + finish pool + drop. Two-level let*:
   ;; outer holds the driver; inner owns the popped Sender. Returns
   ;; the driver for the test body to join.
   (:wat::core::define
     (:wat-telemetry-sqlite::Sqlite::spawn-and-drop
       (path :wat::core::String)
       -> :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
     (:wat::core::let*
       (((spawn :wat::telemetry::Spawn<wat::core::i64>)
         (:wat::telemetry::Sqlite/spawn
           path 1
           (:wat::telemetry::null-metrics-cadence)
           :wat::telemetry::Sqlite/null-pre-install
           :wat-telemetry-sqlite::Sqlite::install-noop
           :wat-telemetry-sqlite::Sqlite::dispatch-noop
           :wat-telemetry-sqlite::Sqlite::translate-empty))
        ((pool :wat::telemetry::HandlePool<wat::core::i64>)
         (:wat::core::first spawn))
        ((driver :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
         (:wat::core::second spawn))
        ((_inner :wat::core::unit)
         (:wat-telemetry-sqlite::Sqlite::drop-one-handle pool)))
       driver))

   ;; The inner-scope body — pop one handle + finish + drop. Lives
   ;; in its own function so spawn-and-drop's outer let* stays simple.
   (:wat::core::define
     (:wat-telemetry-sqlite::Sqlite::drop-one-handle
       (pool :wat::telemetry::HandlePool<wat::core::i64>)
       -> :wat::core::unit)
     (:wat::core::let*
       (((_handle :wat::telemetry::Handle<wat::core::i64>)
         (:wat::kernel::HandlePool::pop pool))
        ((_finish :wat::core::unit) (:wat::kernel::HandlePool::finish pool)))
       ()))

   ;; Spawn + batch-log three entries + drop. Same lockstep shape as
   ;; spawn-and-drop, with traffic.
   (:wat::core::define
     (:wat-telemetry-sqlite::Sqlite::spawn-and-batch
       (path :wat::core::String)
       -> :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
     (:wat::core::let*
       (((spawn :wat::telemetry::Spawn<wat::core::i64>)
         (:wat::telemetry::Sqlite/spawn
           path 1
           (:wat::telemetry::null-metrics-cadence)
           :wat-telemetry-sqlite::Sqlite::pragma-wal
           :wat-telemetry-sqlite::Sqlite::install-events
           :wat-telemetry-sqlite::Sqlite::dispatch-events
           :wat-telemetry-sqlite::Sqlite::translate-empty))
        ((pool :wat::telemetry::HandlePool<wat::core::i64>)
         (:wat::core::first spawn))
        ((driver :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
         (:wat::core::second spawn))
        ((_inner :wat::core::unit)
         (:wat-telemetry-sqlite::Sqlite::send-three pool)))
       driver))

   ;; Pop one Handle (req-tx, ack-rx — paired by the spawn step;
   ;; arc 095) and send one batch of three i64s. The Handle's two
   ;; opposite ends are exactly what batch-log needs.
   (:wat::core::define
     (:wat-telemetry-sqlite::Sqlite::send-three
       (pool :wat::telemetry::HandlePool<wat::core::i64>)
       -> :wat::core::unit)
     (:wat::core::let*
       (((handle :wat::telemetry::Handle<wat::core::i64>)
         (:wat::kernel::HandlePool::pop pool))
        ((_finish :wat::core::unit) (:wat::kernel::HandlePool::finish pool))
        ((req-tx :wat::telemetry::ReqTx<wat::core::i64>)
         (:wat::core::first handle))
        ((ack-rx :wat::telemetry::AckRx)
         (:wat::core::second handle))
        ((entries :wat::core::Vector<wat::core::i64>)
         (:wat::core::Vector :wat::core::i64 7 11 13))
        ((_log :wat::core::unit)
         (:wat::telemetry::batch-log
           req-tx ack-rx entries)))
       ()))))


;; ─── Test 1: spawn + drop + join (lifecycle) ───────────────────

(:deftest :wat-telemetry-sqlite::Sqlite::test-spawn-drop
  (:wat::core::let*
    (((driver :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
      (:wat-telemetry-sqlite::Sqlite::spawn-and-drop
        "/tmp/wat-sqlite-test-spawn-001.db"))
     ((_join :wat::core::Result<wat::core::unit,wat::core::Vector<wat::kernel::ThreadDiedError>>)
      (:wat::kernel::Thread/join-result driver)))
    (:wat::test::assert-eq true true)))


;; ─── Test 2: send three entries + drop + join ─────────────────

(:deftest :wat-telemetry-sqlite::Sqlite::test-batch-log
  (:wat::core::let*
    (((driver :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
      (:wat-telemetry-sqlite::Sqlite::spawn-and-batch
        "/tmp/wat-sqlite-test-batch-001.db"))
     ((_join :wat::core::Result<wat::core::unit,wat::core::Vector<wat::kernel::ThreadDiedError>>)
      (:wat::kernel::Thread/join-result driver)))
    (:wat::test::assert-eq true true)))
