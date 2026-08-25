;; uuid_from_string_valid.wat — canonical lowercase UUID → Some.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [result (:wat::uuid::from-string "550e8400-e29b-41d4-a716-446655440000")]
    (:wat::core::match result 
      ((:wat::core::Some u) (:wat::kernel::println "VALID-SOME"))
      (:wat::core::None     (:wat::kernel::println "VALID-NONE")))))
