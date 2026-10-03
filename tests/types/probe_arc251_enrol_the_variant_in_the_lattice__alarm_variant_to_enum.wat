;; Alarm<Op.Mark> -> Alarm<Op> value slot ACCEPTED (SAME-head lattice, already true pre-stone)
(wat.core/defenum u/Op wat.enum/Pure
  :Mark [n :- wat.type/i64]
  :Other [])
(wat.core/defrecord u/Alarm :- [O] [op :- O])
(wat.core/defn u/takes-enum [a :- (u/Alarm :- [u/Op])] :- wat.type/nil
  (wat.kernel/println "ok"))
(wat.core/defn user/main [] :- wat.type/nil
  (u/takes-enum (u/Alarm :op (u/Op.Mark {:n 1}))))
