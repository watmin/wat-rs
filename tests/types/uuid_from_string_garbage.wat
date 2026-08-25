;; uuid_from_string_garbage.wat — garbage string → None.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [result (:wat::uuid::from-string "not-a-uuid")]
    (:wat::core::match result 
      ((:wat::core::Some u) (:wat::kernel::println "GARBAGE-SOME"))
      (:wat::core::None     (:wat::kernel::println "GARBAGE-NONE")))))
