;; Co-located fixture for wat_run_sandboxed.rs — slurped via startup_beside(file!()).
;; One compute fn per test; all return RunResult (read via substrate accessor fns in Rust).

(:wat::core::defn :my::compute-noop [] -> :wat::kernel::RunResult
  (:wat::test::run-hermetic nil))

(:wat::core::defn :my::compute-single-line [] -> :wat::kernel::RunResult
  (:wat::test::run-hermetic
    (:wat::kernel::println "hello")))

(:wat::core::defn :my::compute-stdout-stderr [] -> :wat::kernel::RunResult
  (:wat::test::run-hermetic
    (:wat::core::do
      (:wat::kernel::println "one")
      (:wat::kernel::println "two")
      (:wat::kernel::eprintln "oops")
      nil)))

(:wat::core::defn :my::compute-parse-error [] -> :wat::kernel::RunResult
  (:wat::test::run-hermetic
    (:wat::kernel::raise! (:wat::core::Fault/of "inner-failure"))))

(:wat::core::defn :my::compute-missing-main [] -> :wat::kernel::RunResult
  (:wat::test::run-hermetic
    (:wat::kernel::raise! (:wat::core::Fault/of "needs-main-sentinel"))))

(:wat::core::defn :my::compute-panic-partial [] -> :wat::kernel::RunResult
  (:wat::test::run-hermetic
    (:wat::core::let
      [_ (:wat::kernel::println "before panic")
       _ (:wat::kernel::raise! (:wat::core::Fault/of "boom"))]
      nil)))

;; Scope tests: under hermetic the child's InMemoryLoader has no entries,
;; so eval-file! always takes the Err arm regardless of the path provided.
(:wat::core::defn :my::compute-scope-inside [] -> :wat::kernel::RunResult
  (:wat::test::run-hermetic
    (:wat::core::match
      (:wat::eval-file! "/nonexistent-in-child-loader.wat")
      -> :wat::core::nil
      ((:wat::core::Ok h) (:wat::kernel::println "ok"))
      ((:wat::core::Err _) (:wat::kernel::eprintln "err")))))

(:wat::core::defn :my::compute-scope-outside [] -> :wat::kernel::RunResult
  (:wat::test::run-hermetic
    (:wat::core::match
      (:wat::eval-file! "/also-nonexistent-in-child-loader.wat")
      -> :wat::core::nil
      ((:wat::core::Ok _) (:wat::kernel::println "leaked"))
      ((:wat::core::Err _) (:wat::kernel::eprintln "blocked")))))

