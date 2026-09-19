;; Default collect-deadline-ms (env unset). Production value, not a lowered one.
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::program::collect-deadline-ms))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:wat::i64::to-string (:user::compute))))
