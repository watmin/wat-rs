;; Fixture probe 01: subtype accepted at a single-arg boundary — must type-check Ok.
(:wat::core::defrecord :my::Circle [radius <- :wat::core::f64])
(:wat::core::defrecord :my::Square [side <- :wat::core::f64])
(:wat::core::defn :my::needs-record [v <- :wat::core::Record] -> :wat::core::f64 1.0)
(:wat::core::defn :my::force [c <- :my::Circle] -> :wat::core::f64 (:my::needs-record c))
