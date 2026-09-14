;; 2a4b — `stdlib-source-path?` is a repo-relative `wat/` prefix, not `contains "/wat/"`.
(:wat::test::deftest :wat-tests::fix::stdlib-source-path-stdlib-file
  (:wat::test::assert-eq (:wat::fix::stdlib-source-path? "wat/core.wat") true))

(:wat::test::deftest :wat-tests::fix::stdlib-source-path-edn-clj-shared-is-not-stdlib
  (:wat::test::assert-eq
    (:wat::fix::stdlib-source-path? "crates/wat-edn/wat-edn-clj/wat/shared.wat")
    false))

(:wat::test::deftest :wat-tests::fix::stdlib-source-path-example-is-not-stdlib
  (:wat::test::assert-eq
    (:wat::fix::stdlib-source-path? "examples/console-demo/wat/main.wat")
    false))
