;; CONTROL — a phantom in ARG position is already refused (the args ARE walked today).
(:wat::core::defn :user::f [x <- (:wat::core::Option :- [:usr::TotallyMadeUp])] -> wat.type/nil
  (:wat::kernel::println "ok"))
(:wat::core::defn :user::main [] -> wat.type/nil (:wat::kernel::println "ok"))
