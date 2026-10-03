;; VARIANT parametric (:has d) -> i64  CORRECT  accept
(wat.core/defenum u/Demo :- [T] wat.enum/Pure
  :Has    [has :- T]
  :HasNot [])
(wat.core/defn u/fb [d :- (u/Demo.Has :- [wat.type/i64])] :- wat.type/i64 (:has d))
(wat.core/defn user/main [] :- wat.type/nil (wat.kernel/println "ok"))
