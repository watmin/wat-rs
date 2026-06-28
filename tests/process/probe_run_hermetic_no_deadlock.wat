;; tests/process/probe_run_hermetic_no_deadlock.wat — co-located fixture for probe_run_hermetic_no_deadlock.rs
;; startup_beside(file!()) world — both probe defns coexist in the same world:
;;   :probe::test::clean-exit        (Probe 1)
;;   :probe::test::intentional-panic (Probe 2)

;; Probe 1 — empty body returning nil; child exits 0; no deadlock.
(:wat::core::defn :probe::test::clean-exit [] -> :wat::kernel::RunResult
  (:wat::test::run-hermetic
    nil))

;; Probe 2 — body calling assertion-failed!; child panics; drain-before-join drains stderr before join.
(:wat::core::defn :probe::test::intentional-panic [] -> :wat::kernel::RunResult
  (:wat::test::run-hermetic
    (:wat::kernel::assertion-failed!
      "intentional panic from probe_run_hermetic_no_deadlock"
      :wat::core::None
      :wat::core::None)))

