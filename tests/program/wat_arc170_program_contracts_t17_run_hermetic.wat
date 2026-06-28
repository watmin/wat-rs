;; tests/program/wat_arc170_program_contracts_t17_run_hermetic.wat — run-hermetic with passing assertion (2+2=4).
(:wat::core::defn :my::test::two-plus-two [] -> :wat::kernel::RunResult
  (:wat::test::run-hermetic
    (:wat::test::assert-eq (:wat::core::i64::+ 2 2) 4)))
