;; RECORD monomorphic (:x c) -> i64  CORRECT  accept
(wat.core/defrecord u/Plain [x :- wat.type/i64])
(wat.core/defn u/c [c :- u/Plain] :- wat.type/i64 (:x c))
(wat.core/defn user/main [] :- wat.type/nil (wat.kernel/println "ok"))
