;; uuid_from_string_braced.wat — braced UUID → None.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [result (:wat::uuid::from-string "{550e8400-e29b-41d4-a716-446655440000}")]
    (:wat::core::match result 
      [:wat::core::Option.Some {:value u} (:wat::kernel::println "BRACED-SOME")]
      [:wat::core::Option.None {}     (:wat::kernel::println "BRACED-NONE")])))
