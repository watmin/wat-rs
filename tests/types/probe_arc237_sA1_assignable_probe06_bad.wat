;; Negative fixture probe 06: unrelated :my::Square into :my::Circle slot — must be a type error (no edge).
(:wat::core::defrecord :my::Circle [radius <- :wat::core::f64])
(:wat::core::defrecord :my::Square [side <- :wat::core::f64])
(:wat::core::defn :needs-circle [c <- :my::Circle] -> :wat::core::f64 1.0)
(:wat::core::defn :feed-sq [s <- :my::Square] -> :wat::core::f64 (:needs-circle s))
(:wat::core::defn :user::main [] -> :wat::core::nil nil)
