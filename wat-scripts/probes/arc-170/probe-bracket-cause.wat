;; Gate 3 — a runner that crashes in the child must surface the REAL cause in the
;; collect-loop assertion message ("runner {idx} crashed: {cause}"), not a blind
;; "runner crashed". The work-fn divides by zero → the child panics → Lost{idx,cause}.
(:wat::core::defn :probe::boom [n <- :wat::core::i64] -> :wat::core::i64
  (:wat::i64::/ n 0))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println
    (:wat::bracket::map (:wat::spawn::process)
      (:wat::core::Vector :- [:wat::core::i64] 1 2 3)
      :probe::boom)))
