;; RUNTIME twin — the correct bare-keyword read RUNS and prints
(:wat::core::defrecord :u::Plain [x <- :wat::core::i64])
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [c (:u::Plain :x 42)]
    (:wat::kernel::println (:x c))))
