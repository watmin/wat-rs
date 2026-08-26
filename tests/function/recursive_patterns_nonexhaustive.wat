;; tests/function/recursive_patterns_nonexhaustive.wat — NEGATIVE: non-exhaustive partial pattern.
;; startup MUST fail with "non-exhaustive" error.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
              [row
                (:wat::core::Some (:wat::core::Tuple 1 2))
               v
                (:wat::core::match row 
                  ((:wat::core::Some (1 x)) x)
                  (:wat::core::None 0))]
              (:wat::kernel::println (:wat::i64::to-string v))))
