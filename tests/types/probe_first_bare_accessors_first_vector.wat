;; first (bare) on a typed (Vector :- [i64]). RED at HEAD: first returns (Option :- [T]).
(:wat::core::defn :p::f [] -> :wat::core::i64 (:wat::core::first (:wat::core::Vector :- [:wat::core::i64] 10 20 30)))
