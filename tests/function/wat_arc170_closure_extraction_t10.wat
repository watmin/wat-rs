;; T10: typealias Coord + defn compute — type alias must land in prologue.
(:wat::core::typealias :my::Coord :wat::core::i64)
(:wat::core::defn :my::compute [c <- :my::Coord] -> :wat::core::i64 (:wat::i64::+ c 1))
