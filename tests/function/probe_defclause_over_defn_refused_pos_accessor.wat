;; NON-VACUITY for the accessor pair: the same program with the clause REMOVED.
(:wat::load-file! "lib-record.wat")
(:wat::core::defn :user::probe [] -> :wat::core::i64
  (:mylib::sum (:mylib::Point :x 3 :y 4)))
