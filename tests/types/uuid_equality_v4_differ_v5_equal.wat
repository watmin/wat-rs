;; uuid_equality_v4_differ_v5_equal.wat — v4 differs; v5 same args equals.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [a  (:wat::uuid::v4)
     b  (:wat::uuid::v4)
     ns (:wat::uuid::nil)
     c  (:wat::uuid::v5 ns "same-name")
     d  (:wat::uuid::v5 ns "same-name")]
    (:wat::core::do
      (:wat::core::if (:wat::core::= a b) 
        (:wat::kernel::println "V4-SAME")
        (:wat::kernel::println "V4-DIFFER"))
      (:wat::core::if (:wat::core::= c d) 
        (:wat::kernel::println "V5-EQUAL")
        (:wat::kernel::println "V5-DIFFER")))))
