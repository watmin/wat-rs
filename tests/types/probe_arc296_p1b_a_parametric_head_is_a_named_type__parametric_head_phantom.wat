;; SUBJECT — the HEAD of a parametric annotation names nothing. Accepted today.
(:wat::core::defn :user::f [x <- (:usr::TotallyMadeUp :- [wat.type/i64])] -> wat.type/nil
  (:wat::kernel::println "ok"))
(:wat::core::defn :user::main [] -> wat.type/nil (:wat::kernel::println "ok"))
