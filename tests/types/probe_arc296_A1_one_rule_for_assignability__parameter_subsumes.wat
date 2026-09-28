;; CONTROL — a parameter position ALREADY subsumes. Green today, must stay green.
(:wat::core::defrecord :usr::Child  [v <- wat.type/i64])
(:wat::core::defrecord :usr::Parent [v <- wat.type/i64])
(:wat::core::derive :usr::Child :usr::Parent)
(:wat::core::defn :user::takes-parent [p <- :usr::Parent] -> wat.type/nil (:wat::kernel::println "ok"))
(:wat::core::defn :user::main [] -> wat.type/nil (:user::takes-parent (:usr::Child 1)))
