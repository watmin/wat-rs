;; Fixture probe 05: transitive :my::Special <: :my::Circle <: :wat::Record — must type-check Ok.
(:wat::core::defrecord :my::Circle [radius <- :wat::core::f64])
(:wat::core::defrecord :my::Square [side <- :wat::core::f64])
(:wat::core::recordtype :my::Special :my::Circle [])
(:wat::core::defn :needs-record [v <- :wat::Record] -> :wat::core::f64 1.0)
(:wat::core::defn :force3 [s <- :my::Special] -> :wat::core::f64 (:needs-record s))
