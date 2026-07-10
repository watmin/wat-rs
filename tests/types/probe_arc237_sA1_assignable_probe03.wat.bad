;; Negative fixture probe 03: supertype into subtype slot — must remain a type error.
(:wat::core::defrecord :my::Circle [radius <- :wat::core::f64])
(:wat::core::defrecord :my::Square [side <- :wat::core::f64])
(:wat::core::defn :needs-circle [c <- :my::Circle] -> :wat::core::f64 1.0)
(:wat::core::defn :feed [r <- :wat::core::Record] -> :wat::core::f64 (:needs-circle r))
