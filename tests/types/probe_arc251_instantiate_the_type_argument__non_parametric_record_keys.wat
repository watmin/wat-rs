;; CONTROL — monomorphic record {:keys} was never the defect.
(:wat::core::defrecord :u::Cell [x <- wat.type/i64])
(:wat::core::defn :u::f [c <- :u::Cell] -> wat.type/i64
  (:wat::core::let [{:keys [x]} c] x))
(:wat::core::defn :user::main [] -> wat.type/nil (:wat::kernel::println "ok"))
