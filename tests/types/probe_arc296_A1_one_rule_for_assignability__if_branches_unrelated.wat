;; ⛔ OVER-REACH DETECTOR — two UNRELATED concrete types must STILL be refused.
;; A stone that made  accept anything would pass the subject and fail here.
(:wat::core::defrecord :usr::Child  [v <- :wat::core::i64])
(:wat::core::defrecord :usr::Parent [v <- :wat::core::i64])
(:wat::core::derive :usr::Child :usr::Parent)
(:wat::core::defn :user::pick [b <- :wat::core::bool] -> :wat::core::nil
  (:wat::core::if b (:usr::Child 1) "a string"))
(:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println "ok"))
