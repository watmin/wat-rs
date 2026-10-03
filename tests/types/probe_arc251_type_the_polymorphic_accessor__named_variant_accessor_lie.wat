;; CONTROL — named VARIANT accessor lie still refused
(wat.core/defenum u/Demo :- [T] wat.enum/Pure
  :Has    [has :- T]
  :HasNot [])
(wat.core/defn u/fa [d :- (u/Demo.Has :- [wat.type/i64])] :- wat.type/String
  (u.Demo.Has/has d))
(wat.core/defn user/main [] :- wat.type/nil (wat.kernel/println "ok"))
