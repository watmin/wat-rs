;; wat-tests/std/telemetry/Sqlite.wat — arc 083 slice 2 smoke tests.
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
;; bodies (precedent: wat-rs/wat-tests/std/service-template.wat).

(:wat::test::make-deftest :deftest
  (;; ─── Hooks (no-ops; lifecycle test) ─────────────────────────

   (:wat::core::define
     (:wat-tests::std::telemetry::Sqlite::install-noop
       (_db :wat::sqlite::Db)
       -> :())
     ())

   (:wat::core::define
     (:wat-tests::std::telemetry::Sqlite::dispatch-noop
       (_db :wat::sqlite::Db)
       (_entry :i64)
       -> :())
     ())

   (:wat::core::define
     (:wat-tests::std::telemetry::Sqlite::translate-empty
       (_stats :wat::std::telemetry::Service::Stats)
       -> :Vec<i64>)
     (:wat::core::vec :i64))


   ;; ─── Hooks (insert; traffic test) ────────────────────────────

   (:wat::core::define
     (:wat-tests::std::telemetry::Sqlite::install-events
       (db :wat::sqlite::Db)
       -> :())
     (:wat::sqlite::execute-ddl db
       "CREATE TABLE IF NOT EXISTS events (n INTEGER)"))

   ;; SQL-string-concat insert is acceptable here — i64 is internally
   ;; typed; no injection vector. A future slice's parameterized
   ;; execute primitive removes the concat.
   (:wat::core::define
     (:wat-tests::std::telemetry::Sqlite::dispatch-events
       (db :wat::sqlite::Db)
       (entry :i64)
       -> :())
     (:wat::core::let*
       (((sql :String)
         (:wat::core::string::concat
           "INSERT INTO events (n) VALUES ("
           (:wat::core::string::concat
             (:wat::core::i64::to-string entry)
             ")"))))
       (:wat::sqlite::execute-ddl db sql)))


   ;; ─── Helpers — function-decomposed lockstep (Step 9) ────────

   ;; Spawn + pop one handle + finish pool + drop. Two-level let*:
   ;; outer holds the driver; inner owns the popped Sender. Returns
   ;; the driver for the test body to join.
   (:wat::core::define
     (:wat-tests::std::telemetry::Sqlite::spawn-and-drop
       (path :String)
       -> :wat::kernel::ProgramHandle<()>)
     (:wat::core::let*
       (((spawn :wat::std::telemetry::Service::Spawn<i64>)
         (:wat::std::telemetry::Sqlite/spawn
           path 1
           (:wat::std::telemetry::Service/null-metrics-cadence)
           :wat-tests::std::telemetry::Sqlite::install-noop
           :wat-tests::std::telemetry::Sqlite::dispatch-noop
           :wat-tests::std::telemetry::Sqlite::translate-empty))
        ((pool :wat::std::telemetry::Service::ReqTxPool<i64>)
         (:wat::core::first spawn))
        ((driver :wat::kernel::ProgramHandle<()>)
         (:wat::core::second spawn))
        ((_inner :())
         (:wat-tests::std::telemetry::Sqlite::drop-one-handle pool)))
       driver))

   ;; The inner-scope body — pop one handle + finish + drop. Lives
   ;; in its own function so spawn-and-drop's outer let* stays simple.
   (:wat::core::define
     (:wat-tests::std::telemetry::Sqlite::drop-one-handle
       (pool :wat::std::telemetry::Service::ReqTxPool<i64>)
       -> :())
     (:wat::core::let*
       (((_tx :wat::std::telemetry::Service::ReqTx<i64>)
         (:wat::kernel::HandlePool::pop pool))
        ((_finish :()) (:wat::kernel::HandlePool::finish pool)))
       ()))

   ;; Spawn + batch-log three entries + drop. Same lockstep shape as
   ;; spawn-and-drop, with traffic.
   (:wat::core::define
     (:wat-tests::std::telemetry::Sqlite::spawn-and-batch
       (path :String)
       -> :wat::kernel::ProgramHandle<()>)
     (:wat::core::let*
       (((spawn :wat::std::telemetry::Service::Spawn<i64>)
         (:wat::std::telemetry::Sqlite/spawn
           path 1
           (:wat::std::telemetry::Service/null-metrics-cadence)
           :wat-tests::std::telemetry::Sqlite::install-events
           :wat-tests::std::telemetry::Sqlite::dispatch-events
           :wat-tests::std::telemetry::Sqlite::translate-empty))
        ((pool :wat::std::telemetry::Service::ReqTxPool<i64>)
         (:wat::core::first spawn))
        ((driver :wat::kernel::ProgramHandle<()>)
         (:wat::core::second spawn))
        ((_inner :())
         (:wat-tests::std::telemetry::Sqlite::send-three pool)))
       driver))

   ;; Pop one handle + build an ack channel + send one batch of three
   ;; i64s + finish + drop. The popped req-tx and the ack pair all
   ;; drop together when this function returns.
   (:wat::core::define
     (:wat-tests::std::telemetry::Sqlite::send-three
       (pool :wat::std::telemetry::Service::ReqTxPool<i64>)
       -> :())
     (:wat::core::let*
       (((req-tx :wat::std::telemetry::Service::ReqTx<i64>)
         (:wat::kernel::HandlePool::pop pool))
        ((_finish :()) (:wat::kernel::HandlePool::finish pool))
        ((ack-pair :wat::std::telemetry::Service::AckChannel)
         (:wat::kernel::make-bounded-queue :() 1))
        ((ack-tx :wat::std::telemetry::Service::AckTx)
         (:wat::core::first ack-pair))
        ((ack-rx :wat::std::telemetry::Service::AckRx)
         (:wat::core::second ack-pair))
        ((entries :Vec<i64>)
         (:wat::core::vec :i64 7 11 13))
        ((_log :())
         (:wat::std::telemetry::Service/batch-log
           req-tx ack-tx ack-rx entries)))
       ()))))


;; ─── Test 1: spawn + drop + join (lifecycle) ───────────────────

(:deftest :wat-tests::std::telemetry::Sqlite::test-spawn-drop
  (:wat::core::let*
    (((driver :wat::kernel::ProgramHandle<()>)
      (:wat-tests::std::telemetry::Sqlite::spawn-and-drop
        "/tmp/wat-sqlite-test-spawn-001.db"))
     ((_join :()) (:wat::kernel::join driver)))
    (:wat::test::assert-eq true true)))


;; ─── Test 2: send three entries + drop + join ─────────────────

(:deftest :wat-tests::std::telemetry::Sqlite::test-batch-log
  (:wat::core::let*
    (((driver :wat::kernel::ProgramHandle<()>)
      (:wat-tests::std::telemetry::Sqlite::spawn-and-batch
        "/tmp/wat-sqlite-test-batch-001.db"))
     ((_join :()) (:wat::kernel::join driver)))
    (:wat::test::assert-eq true true)))
