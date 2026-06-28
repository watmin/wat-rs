;; tests/diagnostics/probe_runtime_error_produces_structured_edn.wat — co-located fixture for the
;; sibling probe (.rs), slurped via startup_beside(file!()).
;;
;; Body: calls division by zero → RuntimeError::DivisionByZero in run-hermetic.
;; Hits Ok(Err(runtime_err)) arm in spawn_process_child_branch.
(:wat::core::defn :probe::runtime-err [] -> :wat::kernel::RunResult
  (:wat::test::run-hermetic
              ;; Division by zero → RuntimeError::DivisionByZero.
              ;; Passes type-check; fails at child runtime.
              ;; Hits Ok(Err(runtime_err)) arm in spawn_process_child_branch.
              (:wat::core::let [_ (:wat::core::i64::/ 1 0)] :wat::core::nil)))
