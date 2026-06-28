;; uuid_eq_uses_values_equal_arm.wat — (= nil-uuid nil-uuid) uses values_equal arm.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [a (:wat::core::Uuid/nil)
     b (:wat::core::Uuid/nil)]
    (:wat::core::if (:wat::core::= a b) -> :wat::core::nil
      (:wat::kernel::println "NIL-EQ-OK")
      (:wat::kernel::println "NIL-EQ-FAIL"))))
