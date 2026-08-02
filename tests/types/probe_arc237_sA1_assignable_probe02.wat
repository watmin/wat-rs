;; Fixture probe 02: subtype accepted at a multi-arg boundary, 2nd param — must type-check Ok.
(:wat::core::defrecord :my::Circle [radius <- :wat::core::f64])
(:wat::core::defrecord :my::Square [side <- :wat::core::f64])
(:wat::core::defn :my::two [a <- :wat::core::f64 b <- :wat::core::Record] -> :wat::core::f64 a)
(:wat::core::defn :my::force2 [c <- :my::Circle] -> :wat::core::f64 (:my::two 2.0 c))
