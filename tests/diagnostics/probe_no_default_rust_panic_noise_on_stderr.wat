;; tests/diagnostics/probe_no_default_rust_panic_noise_on_stderr.wat — co-located fixture for the
;; sibling probe (.rs), slurped via startup_beside(file!()).
;;
;; Body triggers an AssertionPayload panic via assert-eq mismatch.
;; The child's panic hook is installed BEFORE catch_unwind — Rust's default handler
;; (which would write "thread '...' panicked" etc.) is suppressed.
(:wat::core::defn :probe::hook-test [] -> :wat::kernel::RunResult
  (:wat::test::run-hermetic
              (:wat::test::assert-eq "expected-value" "actual-value")))
