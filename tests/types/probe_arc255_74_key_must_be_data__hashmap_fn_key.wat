(wat.core/defn probe/inc [x :- wat.type/i64] :- wat.type/i64 (wat.i64/+ x 1))
(wat.core/defn user/main [] :- wat.type/nil
  (wat.core/let
    [m (wat.type/HashMap :- [[wat.type/i64 :-> wat.type/i64] wat.type/i64] probe/inc 1)]
    (wat.kernel/println "fn key in HashMap")))
