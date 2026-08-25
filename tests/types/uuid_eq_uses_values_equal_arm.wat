;; uuid_eq_uses_values_equal_arm.wat — (= nil-uuid nil-uuid) uses values_equal arm.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [a (:wat::uuid::nil)
     b (:wat::uuid::nil)]
    (:wat::core::if (:wat::core::= a b) 
      (:wat::kernel::println "NIL-EQ-OK")
      (:wat::kernel::println "NIL-EQ-FAIL"))))
