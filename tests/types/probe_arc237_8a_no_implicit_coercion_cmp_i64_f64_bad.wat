;; Negative fixture: i64 < f64 must reject at check (no implicit coercion in comparison).
(:wat::core::defn :user::compute [] -> :wat::core::bool (:wat::core::< 1 2.0))
