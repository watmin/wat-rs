;; One clause per line; body rides its test; bodies align across clauses.
(wat.core/defn fix/cond-align
  [k :- wat.type/i64]
  :- wat.type/String
  (wat.core/cond
    ((wat.i64/= k 0) "zero")
    ((wat.i64/= k 1) "one")
    (:else "many")))
