;; CONTROL — VARIANT {:keys} lie still refused
(wat.core/defenum u/Demo :- [T] wat.enum/Pure
  :Has    [has :- T]
  :HasNot [])
(wat.core/defn u/fv [d :- (u/Demo.Has :- [wat.type/i64])] :- wat.type/String
  (wat.core/let [{:keys [has]} d] has))
(wat.core/defn user/main [] :- wat.type/nil (wat.kernel/println "ok"))
