;; CONTROL — named RECORD accessor lie still refused
(wat.core/defrecord u/Plain [x :- wat.type/i64])
(wat.core/defn u/c [c :- u/Plain] :- wat.type/String (u.Plain/x c))
(wat.core/defn user/main [] :- wat.type/nil (wat.kernel/println "ok"))
