;; uuid_v5_with_typed_namespace.wat — Uuid/v5 with typed namespace is deterministic.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [ns (:wat::uuid::nil)
     u1 (:wat::uuid::v5 ns "hello")
     u2 (:wat::uuid::v5 ns "hello")
     s1 (:wat::uuid::to-string u1)]
    (:wat::core::do
      (:wat::core::if (:wat::core::= (:wat::string::length s1) 36) 
        (:wat::kernel::println "V5-LEN-OK")
        (:wat::kernel::println "V5-LEN-FAIL"))
      (:wat::core::if (:wat::core::= u1 u2) 
        (:wat::kernel::println "V5-DETERMINISTIC-OK")
        (:wat::kernel::println "V5-DETERMINISTIC-FAIL")))))
