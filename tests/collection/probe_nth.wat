;; tests/collection/probe_nth.wat — co-located fixture.
;; Reach-stumble: :wat::core::nth — the positional, TOTAL accessor.

;; Probe: nth returns element at index 1 (value 20)
(:wat::core::defn :t::nth-returns-positional [] -> :wat::core::i64
  (:wat::core::nth (:wat::core::Vector :- [:wat::core::i64] 10 20 30) 1))

;; Probe: nth out-of-range raises (index 9 on 3-element vector)
(:wat::core::defn :t::nth-out-of-range [] -> :wat::core::i64
  (:wat::core::nth (:wat::core::Vector :- [:wat::core::i64] 10 20 30) 9))
