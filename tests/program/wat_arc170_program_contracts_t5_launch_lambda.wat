;; tests/program/wat_arc170_program_contracts_t5_launch_lambda.wat — spawn-process with inline forms.
(:wat::core::defn :my::launch [] -> :wat::kernel::Process<wat::core::i64,wat::core::i64>
  (:wat::kernel::spawn-process
    (:wat::core::forms
      (:wat::core::defn :user::main [] -> :wat::core::nil
        (:wat::core::let
          [n    (:wat::kernel::readln -> :wat::core::i64)
           _out (:wat::kernel::println (:wat::core::i64::* n 2))]
          nil)))))
