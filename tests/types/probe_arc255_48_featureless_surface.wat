;; Stone 255.48 — the rows that must load.
;; An edge admits a record to a featureless surface. A featureful surface
;; still admits a wider record with no edge.

(:wat::core::defsurface :probe::Mark :nature :wat::core::Record :features [])
(:wat::core::defrecord :probe::Item [n <- :wat::core::i64])
(:wat::core::extend-type :probe::Item :probe::Mark)

(:wat::core::defn :probe::take-mark [x <- :probe::Mark] -> :wat::core::i64 1)
(:wat::core::defn :user::admitted [] -> :wat::core::i64
  (:probe::take-mark (:probe::Item :n 1)))

(:wat::core::defsurface :probe::HasN :nature :wat::core::Record
  :features [n <- :wat::core::i64])
(:wat::core::defrecord :probe::Pair [n <- :wat::core::i64  extra <- :wat::core::i64])
(:wat::core::defn :probe::take-n [x <- :probe::HasN] -> :wat::core::i64 1)
(:wat::core::defn :user::width [] -> :wat::core::i64
  (:probe::take-n (:probe::Pair :n 1 :extra 2)))
