;; uuid_string_not_equal_typed.wat — typed Uuid == typed Uuid (same content).
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [u   (:wat::uuid::v4)
     s   (:wat::uuid::to-string u)
     opt (:wat::uuid::from-string s)]
    (:wat::core::match opt 
      ((:wat::core::Some u2)
        (:wat::core::if (:wat::core::= u u2) 
          (:wat::kernel::println "UUID-UUID-EQUAL")
          (:wat::kernel::println "UUID-UUID-DIFFER")))
      (:wat::core::None (:wat::kernel::println "PARSE-NONE")))))
