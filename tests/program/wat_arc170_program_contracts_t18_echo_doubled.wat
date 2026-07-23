;; tests/program/wat_arc170_program_contracts_t18_echo_doubled.wat — run-hermetic-with-io: recv 21, send 42.
(:wat::core::defn :my::test::echo-doubled [] -> :wat::test::RunResultIO<wat::core::i64>
  (:wat::test::run-hermetic-with-io
    :wat::core::i64
    :wat::core::i64
    (:wat::core::Vector :wat::core::i64 21)
    (:wat::core::let
      [n (:wat::kernel::readln )
       _ (:wat::kernel::println (:wat::core::i64::* n 2))]
      nil)))
