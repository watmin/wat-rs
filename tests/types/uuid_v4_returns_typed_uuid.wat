;; uuid_v4_returns_typed_uuid.wat — Uuid/v4 returns typed :wat::core::Uuid.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [u  (:wat::uuid::v4)
     s  (:wat::uuid::to-string u)
     ok (:wat::core::= (:wat::string::length s) 36)]
    (:wat::core::if ok 
      (:wat::kernel::println "TYPED-UUID-OK")
      (:wat::kernel::println "TYPED-UUID-FAIL"))))
