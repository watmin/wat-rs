;; tests/function/recursive_patterns_t3.wat — nested_options_three_levels
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
              [mm
                (:wat::core::Option.Some {:value (:wat::core::Option.Some {:value 42})})
               v
                (:wat::core::match mm 
                  [:wat::core::Option.Some {:value [:wat::core::Option.Some {:value x}]} x]
                  [:wat::core::Option.Some {:value :wat::core::Option.None} -1]
                  [:wat::core::Option.None {} -2]
                  [_ -3])]
              (:wat::kernel::println (:wat::i64::to-string v))))
