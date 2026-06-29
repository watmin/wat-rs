;; Fixture probe 04: exact :wat::core::Record into :wat::core::Record — must type-check Ok (regression).
(:wat::core::defrecord :my::Circle [radius <- :wat::core::f64])
(:wat::core::defrecord :my::Square [side <- :wat::core::f64])
(:wat::core::defn :needs-record [v <- :wat::core::Record] -> :wat::core::f64 1.0)
(:wat::core::defn :passthru [v <- :wat::core::Record] -> :wat::core::f64 (:needs-record v))
