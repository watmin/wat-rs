;; tests/function/recursive_patterns_t7.wat — linear_shadowing (second binding wins)
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
              [row
                (:wat::core::Some (:wat::core::Tuple 5 7))
               v
                (:wat::core::match row 
                  ((:wat::core::Some (x x)) x)
                  (:wat::core::None 0))]
              (:wat::kernel::println (:wat::i64::to-string v))))
