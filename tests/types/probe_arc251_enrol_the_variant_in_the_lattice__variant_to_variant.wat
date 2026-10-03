;; Demo.Has -> Demo.Has slot ACCEPTED
(wat.core/defenum u/Demo :- [T] wat.enum/Pure
  :Has    [has :- T]
  :HasNot [])
(wat.core/defn u/takes-has [d :- (u/Demo.Has :- [wat.type/i64])] :- wat.type/nil
  (wat.kernel/println "ok"))
(wat.core/defn user/main [] :- wat.type/nil
  (u/takes-has (u/Demo.Has {:has 1})))
