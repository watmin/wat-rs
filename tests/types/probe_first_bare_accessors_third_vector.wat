;; third (bare) on a (Vector :- [i64]). RED at HEAD.
(wat.core/defn p/f [] :- wat.type/i64 (wat.core/third (wat.type/Vector :- [wat.type/i64] 10 20 30)))
