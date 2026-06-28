;; Negative fixture: f64 + i64 must reject at check (no implicit coercion).
(:wat::core::defn :user::compute [] -> :wat::core::f64 (:wat::core::+ 1.0 2))
(:wat::core::defn :user::main [] -> :wat::core::nil nil)
