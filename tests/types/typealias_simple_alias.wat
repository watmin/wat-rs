;; typealias_simple_alias.wat — simple non-parametric alias unifies with expansion.
(:wat::core::typealias :my::Amount :wat::core::f64)
(:wat::core::defn :app::double [x <- :my::Amount] -> :my::Amount (:wat::f64::* x 2.0))
(:wat::core::defn :my::compute [] -> :wat::core::f64 (:app::double 21.0))
