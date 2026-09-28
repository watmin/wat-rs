;; first (bare) on a typed (Vector :- [i64]). RED at HEAD: first returns (Option :- [T]).
(:wat::core::defn :p::f [] -> wat.type/i64 (:wat::core::first (wat.type/Vector :- [wat.type/i64] 10 20 30)))
