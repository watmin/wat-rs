;; RETIRED — positional variant construction. ACCEPTED today; must be REFUSED once the flip
;; lands, exactly as `defrecord` already refuses `(Pt 1 2)`. This is the row that proves the
;; migration FINISHED rather than merely ADDED a second spelling.
(:wat::core::defenum :probe::Box :wat::enum::Pure :Full [payload <- :wat::core::i64] :Empty [])
(:wat::core::defn :user::take [b <- :probe::Box] -> :wat::core::i64
  (:wat::core::match b [:probe::Box.Full {:payload p} p] [:probe::Box.Empty {} -1]))
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:wat::core::show (:user::take (:probe::Box.Full 7)))))
