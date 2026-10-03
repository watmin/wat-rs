;; T1: top-level defn, no deps, no captures.
(wat.core/defn my/add-one [n :- wat.type/i64] :- wat.type/i64 (wat.i64/+ n 1))
