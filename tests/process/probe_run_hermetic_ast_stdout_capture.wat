;; tests/process/probe_run_hermetic_ast_stdout_capture.wat — co-located fixture for probe_run_hermetic_ast_stdout_capture.rs
;; startup_beside(file!()) world — run-hermetic-ast captures child stdout written via println.

(:wat::core::defn :probe::ast::capture-stdout [] -> :wat::kernel::RunResult
  (:wat::test::run-hermetic
    (:wat::kernel::println "hello-from-probe")))

