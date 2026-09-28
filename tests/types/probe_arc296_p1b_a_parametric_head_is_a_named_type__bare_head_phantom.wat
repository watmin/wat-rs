;; CONTROL — the SAME name, bare. P-1 already refuses this. The two must agree.
(:wat::core::defn :user::f [x <- :usr::TotallyMadeUp] -> wat.type/nil
  (:wat::kernel::println "ok"))
(:wat::core::defn :user::main [] -> wat.type/nil (:wat::kernel::println "ok"))
