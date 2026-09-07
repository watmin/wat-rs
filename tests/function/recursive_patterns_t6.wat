;; tests/function/recursive_patterns_t6.wat — literal_fallback_to_general_arm
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
              [resp (:wat::core::Ok 418)
               label
                (:wat::core::match resp 
                  [:wat::core::Ok {:value 200} "ok"]
                  [:wat::core::Ok {:value 404} "not found"]
                  [:wat::core::Ok {:value n} (:wat::string::concat "code:" (:wat::i64::to-string n))]
                  [:wat::core::Err {:error msg} msg])]
              (:wat::kernel::println label)))
