;; tests/program/wat_arc170_program_contracts_t4_child.wat
;; Child program for T4 (t4_spawn_process_keyword_fn_round_trips_typed_value):
;; reads one i64 from stdin, prints n+1, returns nil.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
              [n    (:wat::kernel::readln -> :wat::core::i64)
               _out (:wat::kernel::println (:wat::core::i64::+ n 1))]
              nil))
