;; Negative fixture: base-defined record must be REJECTED at :wat::holon::Record param (Liskov).
(:wat::core::defrecord :my::Pt [x <- :wat::core::i64  y <- :wat::core::i64])
(:wat::holon::defrecord :my::HPt [x <- :wat::core::i64  y <- :wat::core::i64])
(:wat::core::defn :wh [v <- :wat::holon::Record] -> :wat::core::bool true)
(:wat::core::defn :gb [p <- :my::Pt] -> :wat::core::bool (:wh p))
