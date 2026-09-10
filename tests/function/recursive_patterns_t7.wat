;; tests/function/recursive_patterns_t7.wat — linear_shadowing (second binding wins)
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
              [row
                (:wat::core::Option.Some {:value (:wat::core::Tuple 5 7)})
               v
                (:wat::core::match row 
                  [:wat::core::Option.Some {:value (x x)} x]
                  [:wat::core::Option.None {} 0])]
              (:wat::kernel::println (:wat::i64::to-string v))))
