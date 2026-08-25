;; uuid_from_string_upper.wat — uppercase UUID → None.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [result (:wat::uuid::from-string "550E8400-E29B-41D4-A716-446655440000")]
    (:wat::core::match result 
      ((:wat::core::Some u) (:wat::kernel::println "UPPER-SOME"))
      (:wat::core::None     (:wat::kernel::println "UPPER-NONE")))))
