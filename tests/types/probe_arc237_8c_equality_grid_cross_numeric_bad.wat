;; Negative fixture: cross-numeric = must be a check error (THE DECISION).
(:wat::core::defn :user::compute [] -> :wat::core::bool (:wat::core::= 1 2.0))
(:wat::core::defn :user::main [] -> :wat::core::nil nil)
