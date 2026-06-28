;; tests/diagnostics/probe_runtime_err_stderr_visibility.wat — co-located fixture for the
;; sibling probe (.rs), slurped via startup_beside(file!()).
;;
;; Control case: body triggers an assertion failure via assert-eq mismatch in run-hermetic.
;; Surfaces the full RunResult (stderr + failure) for gap analysis.
(:wat::core::defn :probe::structured [] -> :wat::kernel::RunResult
  (:wat::test::run-hermetic
              (:wat::test::assert-eq "intentional" "different")))
