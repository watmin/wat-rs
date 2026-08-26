;; typealias_alias_chain.wat — alias-of-alias chain expands to root.
(:wat::core::typealias :my::B :wat::core::f64)
(:wat::core::typealias :my::A :my::B)
(:wat::core::defn :app::inc [x <- :my::A] -> :my::A (:wat::f64::+ x 1.0))
(:wat::core::defn :my::compute [] -> :wat::core::f64 (:app::inc 41.0))
