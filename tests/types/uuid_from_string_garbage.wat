;; uuid_from_string_garbage.wat — garbage string → None.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [result (:wat::uuid::from-string "not-a-uuid")]
    (:wat::core::match result 
      [:wat::core::Option::Some {:value u} (:wat::kernel::println "GARBAGE-SOME")]
      [:wat::core::Option::None {}     (:wat::kernel::println "GARBAGE-NONE")])))
