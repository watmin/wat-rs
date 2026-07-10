;; BASELINE: a NON-parametric surface + extend-type + method-call. If this passes, the mechanics
;; work and the <T> param is the gap. If the receiver still fails, my extend-type syntax is wrong.
(:wat::core::defsurface :probe::Holds2 :nature :wat::core::Struct
  :features [(get [self <- :probe::Holds2] -> :wat::core::i64)])
(:wat::core::defrecord :probe::IntBox2 [n <- :wat::core::i64])
(:wat::core::extend-type :probe::IntBox2 :probe::Holds2
  (get [self] (:probe::IntBox2/n self)))
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [b  (:probe::IntBox2 42)
     ok (:wat::core::ann-form (:probe::Holds2/get b) :wat::core::i64)]
    (:wat::kernel::println "measured")))
