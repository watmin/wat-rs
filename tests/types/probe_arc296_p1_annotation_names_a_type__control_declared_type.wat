;; CONTROL — a declared user type in both positions. GREEN now, must stay green.
(:wat::core::defrecord :usr::Point [x <- wat.type/i64 y <- wat.type/i64])
(:wat::core::defn :user::f [p <- :usr::Point] -> :usr::Point p)
(:wat::core::defn :user::main [] -> wat.type/nil (:wat::kernel::println "ok"))
