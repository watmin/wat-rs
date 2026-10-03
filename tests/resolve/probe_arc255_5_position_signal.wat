;; 255.5 — known types in annotation / return position, clojure spelling.
;; Call-position control lives in the sibling .wat.bad.
(wat.core/defn user/id [x :- wat.type/AST] :- wat.type/AST
  x)
(wat.core/defn user/add1 [n :- wat.type/i64] :- wat.type/i64
  (wat.core/+ n 1))
(wat.core/defn user/hold [t :- wat.time/Instant] :- wat.time/Instant
  t)
(wat.core/defn user/main [] :- wat.type/nil
  (wat.kernel/println "ok"))
