;; Negative fixture: cross-numeric = must be a check error.
(:wat::core::defn :user::compute [] -> :wat::core::bool (:wat::core::= 1 2.0))
