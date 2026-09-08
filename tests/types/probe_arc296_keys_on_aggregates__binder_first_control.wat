;; CONTROL — the binder-first form, which ALREADY works on every kind. If this ever
;; fails, the harness is broken and no verdict below means anything.
(:wat::core::defrecord :probe::Pt [x <- :wat::core::i64  y <- :wat::core::i64])
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [pt (:probe::Pt :x 1 :y 2)  {x :x  y :y} pt]
    (:wat::kernel::println (:wat::core::show (:wat::i64::+ x y)))))
