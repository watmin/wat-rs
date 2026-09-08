;; {:keys …} must work on EVERY aggregate kind — they differ in PURITY, not SHAPE.
(:wat::core::defrecord :probe::Pt [x <- :wat::core::i64  y <- :wat::core::i64])
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [pt (:probe::Pt :x 1 :y 2)  {:keys [x y]} pt]
    (:wat::kernel::println (:wat::core::show (:wat::i64::+ x y)))))
