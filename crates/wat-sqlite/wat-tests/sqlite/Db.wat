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


;; ─── Test 3: execute with parameter binding (arc 084) ──────────

(:wat::test::deftest :wat-tests::sqlite::Db::test-execute-params
  ()
  (:wat::core::let*
    (((db :wat::sqlite::Db)
      (:wat::sqlite::open "/tmp/wat-sqlite-test-003.db"))
     ((_create :())
      (:wat::sqlite::execute-ddl db
        "CREATE TABLE IF NOT EXISTS rows (
           run_name  TEXT NOT NULL,
           paper_id  INTEGER NOT NULL,
           residue   REAL NOT NULL,
           ok        INTEGER NOT NULL
         );"))
     ((_clear :())
      (:wat::sqlite::execute-ddl db "DELETE FROM rows;"))
     ((_insert :())
      (:wat::sqlite::execute db
        "INSERT INTO rows (run_name, paper_id, residue, ok) VALUES (?1, ?2, ?3, ?4)"
        (:wat::core::vec :wat::sqlite::Param
          (:wat::sqlite::Param::Str "alpha-run")
          (:wat::sqlite::Param::I64 42)
          (:wat::sqlite::Param::F64 0.125)
          (:wat::sqlite::Param::Bool true)))))
    (:wat::test::assert-eq true true)))
