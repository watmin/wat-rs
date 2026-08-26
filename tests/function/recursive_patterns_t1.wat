;; tests/function/recursive_patterns_t1.wat — option_tuple_single_level_works
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
              [row
                (:wat::core::Some (:wat::core::Tuple 1 2 3))
               sum
                (:wat::core::match row 
                  ((:wat::core::Some (a b c)) (:wat::core::+ a (:wat::core::+ b c)))
                  (:wat::core::None 0))]
              (:wat::kernel::println (:wat::i64::to-string sum))))
