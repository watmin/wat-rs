;; 255.5 — known types in annotation / return position, clojure spelling.
;; Call-position control lives in the sibling .wat.bad.
(:wat::core::defn :user::id [x :- wat/WatAST] -> wat/WatAST
  x)
(:wat::core::defn :user::add1 [n :- wat.core/i64] -> wat.core/i64
  (:wat::core::+ n 1))
(:wat::core::defn :user::hold [t :- wat.time/Instant] -> wat.time/Instant
  t)
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println "ok"))
