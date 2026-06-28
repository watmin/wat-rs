;; tests/program/wat_arc170_program_contracts_t17b_run_hermetic_fail.wat — run-hermetic with failing assertion (1+0 != 2).
(:wat::core::defn :my::test::one-neq-two [] -> :wat::kernel::RunResult
  (:wat::test::run-hermetic
    (:wat::test::assert-eq (:wat::core::i64::+ 1 0) 2)))
