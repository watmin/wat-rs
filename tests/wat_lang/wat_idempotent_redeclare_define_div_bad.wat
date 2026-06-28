;; Negative fixture: divergent defn (same name, different body) → Duplicate error.
;; Used by test: define_divergent_body_errors

(:wat::core::defn :my::add-one [a <- :wat::core::i64] -> :wat::core::i64 (:wat::core::+ a 1))
(:wat::core::defn :my::add-one [a <- :wat::core::i64] -> :wat::core::i64 (:wat::core::+ a 2))
