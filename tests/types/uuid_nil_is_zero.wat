;; uuid_nil_is_zero.wat — Uuid/nil returns the all-zeros UUID.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [u (:wat::uuid::nil)
     s (:wat::uuid::to-string u)]
    (:wat::core::if (:wat::core::= s "00000000-0000-0000-0000-000000000000") 
      (:wat::kernel::println "NIL-OK")
      (:wat::kernel::println "NIL-FAIL"))))
