;; RECORD parametric (:x c) -> String  a lie  REFUSE
(wat.core/defrecord u/Cell :- [X] [x :- X])
(wat.core/defn u/a [c :- (u/Cell :- [wat.type/i64])] :- wat.type/String (:x c))
(wat.core/defn user/main [] :- wat.type/nil (wat.kernel/println "ok"))
