;; tests/program/wat_arc170_slice_1e_user_main_nil_argv.wat — :user::main that reads (:wat::runtime::argv).
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [argv (:wat::runtime::argv)]
    nil))
