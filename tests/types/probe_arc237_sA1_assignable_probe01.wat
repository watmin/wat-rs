;; Fixture probe 01: subtype accepted at a single-arg boundary — must type-check Ok.
(:wat::core::defrecord :my::Circle [radius <- :wat::core::f64])
(:wat::core::defrecord :my::Square [side <- :wat::core::f64])
(:wat::core::defn :needs-record [v <- :wat::Record] -> :wat::core::f64 1.0)
(:wat::core::defn :force [c <- :my::Circle] -> :wat::core::f64 (:needs-record c))
