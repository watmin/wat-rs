;; tests/wat_lang/wat_arc143_define_alias.wat — co-located fixture.
;; Arc 143 slice 6 — alias binding via :wat::core::defalias.

;; Test 1: alias :wat::core::foldl — native registration resolves builtin.
;; Call (:t::my-fold fn 0 vec) → i64 sum.
(:wat::core::defalias :t::my-fold :wat::core::foldl)

(:wat::core::defn :t::test1-foldl-alias [] -> :wat::core::i64
  (:t::my-fold
    (:wat::core::fn
      [acc <- :wat::core::i64 x <- :wat::core::i64] -> :wat::core::i64
      (:wat::core::+ acc x))
    0
    (:wat::core::Vector :- [:wat::core::i64] 1 2 3 4)))

;; Test 2: alias :wat::core::length — verifies native form for multiple targets.
;; Call (:t::my-size vec) → i64 3.
(:wat::core::defalias :t::my-size :wat::core::length)

(:wat::core::defn :t::test2-length-alias [] -> :wat::core::i64
  (:t::my-size
    (:wat::core::Vector :- [:wat::core::i64] 10 20 30)))
