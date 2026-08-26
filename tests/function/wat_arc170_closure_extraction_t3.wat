;; T3: top-level defn uses user types.
(:wat::core::defstruct :my::Point
  [x <- :wat::core::i64
   y <- :wat::core::i64])
(:wat::core::defenum :my::Side :wat::enum::Pure
  :Left
  :Right)
(:wat::core::newtype :my::PriceUsd :wat::core::f64)
(:wat::core::typealias :my::Coord :wat::core::i64)
(:wat::core::defn :my::compute [p <- :my::Point] -> :wat::core::i64 (:wat::i64::+ (:my::Point/x p) (:my::Point/y p)))
