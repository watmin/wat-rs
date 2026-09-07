;; TARGET — a user enum's payload variant built with a MAP naming its declared field.
;; The slot is TYPED, which is what makes this row able to discriminate at all.
(:wat::core::defenum :probe::Box :wat::enum::Pure :Full [payload <- :wat::core::i64] :Empty [])
(:wat::core::defn :user::take [b <- :probe::Box] -> :wat::core::i64
  (:wat::core::match b [:probe::Box::Full {:payload p} p] [:probe::Box::Empty {} -1]))
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:wat::core::show (:user::take (:probe::Box::Full {:payload 7})))))
