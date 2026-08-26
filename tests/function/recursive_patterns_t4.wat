;; tests/function/recursive_patterns_t4.wat — wildcard_at_depth
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
              [row
                (:wat::core::Some (:wat::core::Tuple 100 99 98))
               mid
                (:wat::core::match row 
                  ((:wat::core::Some (_ x _)) x)
                  (:wat::core::None 0))]
              (:wat::kernel::println (:wat::i64::to-string mid))))
