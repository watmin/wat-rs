;; tests/function/recursive_patterns_t5.wat — literal_at_depth_picks_arm
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
              [resp (:wat::core::Ok 200)
               label
                (:wat::core::match resp 
                  ((:wat::core::Ok 200) "ok")
                  ((:wat::core::Ok 404) "not found")
                  ((:wat::core::Ok n) (:wat::string::concat "code:" (:wat::i64::to-string n)))
                  ((:wat::core::Err msg) msg))]
              (:wat::kernel::println label)))
