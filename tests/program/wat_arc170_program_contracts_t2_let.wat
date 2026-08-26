;; tests/program/wat_arc170_program_contracts_t2_let.wat — main with a let-body (i64::+ 1 2).
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [_ (:wat::i64::+ 1 2)]
    nil))
