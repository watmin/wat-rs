;; Co-located fixture for probe_arc278_sqlite_interop.rs — arc 278 stone S1 acceptance gate.
;;
;; Proves the baked :wat::sqlite' RAW interop (a fresh, thread-owned, errors-as-values rusqlite
;; binding — src/rust_deps/sqlite.rs + wat/sqlite.wat) actually round-trips against a REAL sqlite
;; backend: open ":memory:" -> execute-ddl a table -> execute an insert (with Params) -> select it
;; back (assert the Cells) -> force a Constraint fault (insert the same PK twice) -> assert
;; Error::Constraint. Also proves the errors-as-values mechanism end-to-end on a genuinely bad
;; path (open a nonexistent directory) -> assert Error::Fatal, never a panic.

(:wat::core::use! :rust::sqlite::Connection)

(:wat::test::deftest :user::sqlite_interop 
  (:wat::core::let
    [conn (:wat::core::Result/expect (:wat::sqlite::open ":memory:") "open :memory: failed")
     _ddl (:wat::core::Result/expect
            (:wat::sqlite::execute-ddl conn "CREATE TABLE t (id INTEGER PRIMARY KEY, v TEXT)")
            "execute-ddl failed")
     n    (:wat::core::Result/expect
            (:wat::sqlite::execute conn "INSERT INTO t (id, v) VALUES (?, ?)"
              (:wat::core::Vector :- [:wat::sqlite::Param]
                (:wat::sqlite::Param::I64 1) (:wat::sqlite::Param::Str "hello")))
            "insert failed")
     rows (:wat::core::Result/expect
            (:wat::sqlite::select conn "SELECT id, v FROM t ORDER BY id"
              (:wat::core::Vector :- [:wat::sqlite::Param]))
            "select failed")
     row1 (:wat::core::first rows)
     dup  (:wat::sqlite::execute conn "INSERT INTO t (id, v) VALUES (?, ?)"
            (:wat::core::Vector :- [:wat::sqlite::Param]
              (:wat::sqlite::Param::I64 1) (:wat::sqlite::Param::Str "dup")))
     bad  (:wat::sqlite::open "/nonexistent-dir-arc278/x.db")]

    (:wat::test::assert-eq n 1)
    (:wat::test::assert-eq (:wat::core::count rows) 1)
    (:wat::test::assert-eq (:wat::core::count row1) 2)
    (:wat::test::assert-eq (:wat::core::first row1) (:wat::sqlite::Cell::I64 1))
    (:wat::test::assert-eq (:wat::core::second row1) (:wat::sqlite::Cell::Str "hello"))

    (:wat::test::assert-eq
      (:wat::core::match dup 
        ((:wat::core::Err (:wat::sqlite::Error::Constraint _)) true)
        (_ false))
      true)

    (:wat::test::assert-eq
      (:wat::core::match bad 
        ((:wat::core::Err (:wat::sqlite::Error::Fatal _)) true)
        (_ false))
      true)))
