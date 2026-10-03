;; Reverse — fn(Alarm<Op.Mark>) does NOT stand in where fn(Alarm<Op>) is expected.
(wat.core/defenum u/Op wat.enum/Pure
  :Mark [n :- wat.type/i64]
  :Other [])
(wat.core/defrecord u/Alarm :- [O] [op :- O])
(wat.core/defn u/apply
  [f :- [(u/Alarm :- [u/Op]) :-> wat.type/nil]
   a :- (u/Alarm :- [u/Op])] :- wat.type/nil
  (f a))
(wat.core/defn u/narrow [a :- (u/Alarm :- [u/Op.Mark])] :- wat.type/nil
  (wat.kernel/println "ok"))
(wat.core/defn user/main [] :- wat.type/nil
  (u/apply u/narrow (u/Alarm :op (u/Op.Mark {:n 1}))))
