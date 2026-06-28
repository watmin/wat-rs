;; Negative fixture: i64 + f64 must reject at check (no implicit coercion).
(:wat::core::defn :user::compute [] -> :wat::core::f64 (:wat::core::+ 1 2.0))
(:wat::core::defn :user::main [] -> :wat::core::nil nil)
