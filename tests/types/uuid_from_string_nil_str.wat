;; uuid_from_string_nil_str.wat — nil UUID in canonical form → Some.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [result (:wat::uuid::from-string "00000000-0000-0000-0000-000000000000")]
    (:wat::core::match result 
      [:wat::core::Option::Some {:value u} (:wat::kernel::println "NIL-STR-SOME")]
      [:wat::core::Option::None {}     (:wat::kernel::println "NIL-STR-NONE")])))
