;; CONTROL — the SAME name, bare. P-1 already refuses this. The two must agree.
(:wat::core::defn :user::f [x <- :usr::TotallyMadeUp] -> :wat::core::nil
  (:wat::kernel::println "ok"))
(:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println "ok"))
