;; tests/program/wat_arc170_program_contracts_t18b_recv_assert_fail.wat — run-hermetic-with-io: recv 2, assert 3 fails.
(:wat::core::defn :my::test::recv-assert-fail [] -> :wat::test::RunResultIO<wat::core::i64>
  (:wat::test::run-hermetic-with-io
    :wat::core::i64
    :wat::core::i64
    (:wat::core::Vector :wat::core::i64 2)
    (:wat::core::let
      [n (:wat::kernel::readln )
       ;; assert-eq: n=2 vs expected=3 — this fails, child panics
       _ (:wat::test::assert-eq n 3)
       ;; println never reached:
       _2 (:wat::kernel::println n)]
      nil)))
