;; tests/function/recursive_patterns_t9.wat — wildcard_fallback_compiles_and_runs
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
              [row
                (:wat::core::Some (:wat::core::Tuple 1 99))
               v
                (:wat::core::match row 
                  ((:wat::core::Some (1 x)) x)
                  (_ 0))]
              (:wat::kernel::println (:wat::i64::to-string v))))
