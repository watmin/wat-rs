;; first (bare) on a PersistentVector. RED at HEAD.
(wat.core/defn p/f [] :- wat.type/i64 (wat.core/first (wat.type/PersistentVector :- [wat.type/i64] 10 20 30)))
