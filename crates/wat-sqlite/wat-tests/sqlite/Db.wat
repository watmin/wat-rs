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
     ((_ :wat::core::unit)
      (:wat::sqlite::execute-ddl db
        "CREATE TABLE IF NOT EXISTS events (id INTEGER, ts INTEGER)")))
    (:wat::test::assert-eq true true)))


;; ─── Test 3: execute with parameter binding (arc 084) ──────────

(:wat::test::deftest :wat-tests::sqlite::Db::test-execute-params
  ()
  (:wat::core::let*
    (((db :wat::sqlite::Db)
      (:wat::sqlite::open "/tmp/wat-sqlite-test-003.db"))
     ((_create :wat::core::unit)
      (:wat::sqlite::execute-ddl db
        "CREATE TABLE IF NOT EXISTS rows (
           run_name  TEXT NOT NULL,
           paper_id  INTEGER NOT NULL,
           residue   REAL NOT NULL,
           ok        INTEGER NOT NULL
         );"))
     ((_clear :wat::core::unit)
      (:wat::sqlite::execute-ddl db "DELETE FROM rows;"))
     ((_insert :wat::core::unit)
      (:wat::sqlite::execute db
        "INSERT INTO rows (run_name, paper_id, residue, ok) VALUES (?1, ?2, ?3, ?4)"
        (:wat::core::vec :wat::sqlite::Param
          (:wat::sqlite::Param::Str "alpha-run")
          (:wat::sqlite::Param::I64 42)
          (:wat::sqlite::Param::F64 0.125)
          (:wat::sqlite::Param::Bool true)))))
    (:wat::test::assert-eq true true)))


;; ─── Test 4: pragma sets WAL (arc 089 slice 1) ─────────────────
;;
;; Smoke test that `pragma` doesn't crash when setting a real
;; pragma. Verification of journal_mode=WAL on disk happens
;; out-of-band via sqlite3 CLI on the produced db file (look for
;; -wal / -shm sidecar files after running). We can't observe
;; pragma values from wat without the read form (deferred per
;; arc 089 DESIGN).

(:wat::test::deftest :wat-tests::sqlite::Db::test-pragma-wal
  ()
  (:wat::core::let*
    (((db :wat::sqlite::Db)
      (:wat::sqlite::open "/tmp/wat-sqlite-test-004.db"))
     ((_p :wat::core::unit)
      (:wat::sqlite::pragma db "journal_mode" "WAL"))
     ((_p2 :wat::core::unit)
      (:wat::sqlite::pragma db "synchronous" "NORMAL"))
     ((_create :wat::core::unit)
      (:wat::sqlite::execute-ddl db
        "CREATE TABLE IF NOT EXISTS smoke (n INTEGER NOT NULL);")))
    (:wat::test::assert-eq true true)))


;; ─── Test 5: begin/commit wraps inserts (arc 089 slice 1) ─────
;;
;; Open db, set WAL, create a counter table, run begin →
;; three inserts → commit. Same panic-on-error posture as
;; execute-ddl; success means the transaction round-tripped.

(:wat::test::deftest :wat-tests::sqlite::Db::test-begin-commit
  ()
  (:wat::core::let*
    (((db :wat::sqlite::Db)
      (:wat::sqlite::open "/tmp/wat-sqlite-test-005.db"))
     ((_p :wat::core::unit)
      (:wat::sqlite::pragma db "journal_mode" "WAL"))
     ((_create :wat::core::unit)
      (:wat::sqlite::execute-ddl db
        "CREATE TABLE IF NOT EXISTS counters (n INTEGER NOT NULL);"))
     ((_clear :wat::core::unit)
      (:wat::sqlite::execute-ddl db "DELETE FROM counters;"))
     ((_b :wat::core::unit) (:wat::sqlite::begin db))
     ((_i1 :wat::core::unit)
      (:wat::sqlite::execute db
        "INSERT INTO counters (n) VALUES (?1)"
        (:wat::core::vec :wat::sqlite::Param
          (:wat::sqlite::Param::I64 1))))
     ((_i2 :wat::core::unit)
      (:wat::sqlite::execute db
        "INSERT INTO counters (n) VALUES (?1)"
        (:wat::core::vec :wat::sqlite::Param
          (:wat::sqlite::Param::I64 2))))
     ((_i3 :wat::core::unit)
      (:wat::sqlite::execute db
        "INSERT INTO counters (n) VALUES (?1)"
        (:wat::core::vec :wat::sqlite::Param
          (:wat::sqlite::Param::I64 3))))
     ((_c :wat::core::unit) (:wat::sqlite::commit db)))
    (:wat::test::assert-eq true true)))
