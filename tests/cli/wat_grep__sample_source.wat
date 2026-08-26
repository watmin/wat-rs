;; wat_grep__sample_source.wat — a TARGET file (not a --grep rule program). Fixture for G1/G2:
;; enough nameable and unnameable nodes to make Span==Node and Named<Node non-vacuous.
(:wat::core::defn :user::add
  [x <- :wat::core::i64
   y <- :wat::core::i64]
  -> :wat::core::i64
  (:wat::i64::+ x y))
