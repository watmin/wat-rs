;; tests/function/recursive_patterns_t1.wat — option_tuple_single_level_works
(:wat::core::defn :user::main [] -> wat.type/nil
  (:wat::core::let
              [row
                (:wat::core::Option.Some {:value (wat.type/Tuple :- [wat.type/i64 wat.type/i64 wat.type/i64] 1 2 3)})
               sum
                (:wat::core::match row 
                  [:wat::core::Option.Some {:value (a b c)} (:wat::core::+ a (:wat::core::+ b c))]
                  [:wat::core::Option.None {} 0])]
              (:wat::kernel::println (:wat::i64::to-string sum))))
