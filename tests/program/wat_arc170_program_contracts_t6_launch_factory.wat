;; tests/program/wat_arc170_program_contracts_t6_launch_factory.wat — spawn-process with quasiquote + unquote.
(:wat::core::defn :my::launch [offset <- :wat::core::i64] -> :wat::kernel::Process<wat::core::i64,wat::core::i64>
  (:wat::core::let
    [main-form `(:wat::core::defn :user::main [] -> :wat::core::nil
                  (:wat::core::let
                    [n    (:wat::kernel::readln )
                     _out (:wat::kernel::println
                            (:wat::core::i64::+ n ~offset))]
                    nil))]
    (:wat::kernel::spawn-process
      (:wat::core::Vector :wat::WatAST main-form))))
