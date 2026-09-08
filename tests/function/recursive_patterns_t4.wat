;; tests/function/recursive_patterns_t4.wat — wildcard_at_depth
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
              [row
                (:wat::core::Option::Some {:value (:wat::core::Tuple 100 99 98)})
               mid
                (:wat::core::match row 
                  [:wat::core::Option::Some {:value (_ x _)} x]
                  [:wat::core::Option::None {} 0])]
              (:wat::kernel::println (:wat::i64::to-string mid))))
