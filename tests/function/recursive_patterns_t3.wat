;; tests/function/recursive_patterns_t3.wat — nested_options_three_levels
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
              [mm
                (:wat::core::Some (:wat::core::Some 42))
               v
                (:wat::core::match mm 
                  ((:wat::core::Some (:wat::core::Some x)) x)
                  ((:wat::core::Some :wat::core::None) -1)
                  (:wat::core::None -2)
                  (_ -3))]
              (:wat::kernel::println (:wat::i64::to-string v))))
