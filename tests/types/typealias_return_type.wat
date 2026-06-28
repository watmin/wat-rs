;; typealias_return_type.wat — alias in return position unifies with expansion.
(:wat::core::typealias :my::Amount :wat::core::f64)
(:wat::core::defn :app::zero [] -> :my::Amount 0.0)
(:wat::core::defn :my::compute [] -> :wat::core::f64 (:app::zero))
