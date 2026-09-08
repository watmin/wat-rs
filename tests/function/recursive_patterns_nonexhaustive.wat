;; tests/function/recursive_patterns_nonexhaustive.wat — NEGATIVE: non-exhaustive partial pattern.
;; startup MUST fail with "non-exhaustive" error.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
              [row
                (:wat::core::Option::Some {:value (:wat::core::Tuple 1 2)})
               v
                (:wat::core::match row 
                  [:wat::core::Option::Some {:value (1 x)} x]
                  [:wat::core::Option::None {} 0])]
              (:wat::kernel::println (:wat::i64::to-string v))))
