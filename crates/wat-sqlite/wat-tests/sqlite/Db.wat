;; wat-tests/sqlite/Db.wat — arc 083 slice 1 smoke tests.
;;
;; Three deftests covering the substrate sqlite primitives:
;;   - open creates a fresh Db handle (bad path panics; tested
;;     out-of-band — assert by NOT panicking on a /tmp path).
;;   - execute-ddl creates a table successfully.
;;   - execute binds positional params + writes rows.
;;
;; Verification beyond "no crash" happens out-of-band via sqlite3
;; CLI on /tmp/wat-sqlite-*.db. Per the existing rundb pattern.

;; ─── Test 1: open + drop (lifecycle) ─────────────────────────────

(:wat::test::deftest :wat-tests::sqlite::Db::test-open-drop
  ()
  (:wat::core::let*
    (((db :wat::sqlite::Db)
      (:wat::sqlite::open "/tmp/wat-sqlite-test-001.db")))
    (:wat::test::assert-eq true true)))


;; ─── Test 2: execute-ddl creates a schema ───────────────────────

(:wat::test::deftest :wat-tests::sqlite::Db::test-execute-ddl
  ()
  (:wat::core::let*
    (((db :wat::sqlite::Db)
      (:wat::sqlite::open "/tmp/wat-sqlite-test-002.db"))
     ((_ :())
      (:wat::sqlite::execute-ddl db
        "CREATE TABLE IF NOT EXISTS events (id INTEGER, ts INTEGER)")))
    (:wat::test::assert-eq true true)))


;; (Test 3 — parameterized execute — defers to a future slice
;; once the :wat::sqlite::Param enum + Vec<enum> dispatch surface
;; settle. Slice 1 ships open + execute-ddl only.)
