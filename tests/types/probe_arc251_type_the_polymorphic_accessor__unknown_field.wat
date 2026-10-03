;; (:nosuchfield r) on a known receiver  REFUSE, located
(wat.core/defrecord u/Plain [x :- wat.type/i64])
(wat.core/defn u/c [c :- u/Plain] :- wat.type/i64 (:nonexistent c))
(wat.core/defn user/main [] :- wat.type/nil (wat.kernel/println "ok"))
