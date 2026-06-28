;; Negative fixture: divergent typealias (same name, different type) → Duplicate error.
;; Used by test: typealias_divergent_errors

(:wat::core::typealias :my::Amount :wat::core::f64)
(:wat::core::typealias :my::Amount :wat::core::i64)
(:wat::core::defn :t::main [] -> :wat::core::nil nil)
