;; first (bare) on a List. RED at HEAD.
(wat.core/defn p/f [] :- wat.type/i64 (wat.core/first (wat.type/List :- [wat.type/i64] 10 20 30)))
