;; uuid_from_string_urn.wat — URN-prefixed UUID → None.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [result (:wat::uuid::from-string "urn:uuid:550e8400-e29b-41d4-a716-446655440000")]
    (:wat::core::match result 
      ((:wat::core::Some u) (:wat::kernel::println "URN-SOME"))
      (:wat::core::None     (:wat::kernel::println "URN-NONE")))))
