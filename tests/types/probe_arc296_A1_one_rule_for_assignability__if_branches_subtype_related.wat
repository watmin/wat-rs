;; SUBJECT — the SAME pair of types at an . Refused today:
;; ":wat::core::if: parameter else-branch expects :usr::Child; got :usr::Parent"
(:wat::core::defrecord :usr::Child  [v <- :wat::core::i64])
(:wat::core::defrecord :usr::Parent [v <- :wat::core::i64])
(:wat::core::derive :usr::Child :usr::Parent)
(:wat::core::defn :user::pick [b <- :wat::core::bool] -> :usr::Parent
  (:wat::core::if b (:usr::Child 1) (:usr::Parent 2)))
(:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println "ok"))
