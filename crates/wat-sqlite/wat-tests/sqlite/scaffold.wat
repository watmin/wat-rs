;; wat-tests/sqlite/scaffold.wat — slice 0 placeholder.
;;
;; Arc 083 slice 0 ships the crate scaffold without surfaces.
;; This deftest asserts true; slice 1 replaces it with real tests
;; for `:wat::sqlite::Db` (open + execute-ddl + execute).

(:wat::test::deftest :wat-tests::sqlite::test-slice-0-scaffold
  ()
  (:wat::test::assert-eq true true))
