;; ⛔ THE DIRECTION CONTROL — a subtype, not an alias. An arbitrary Box is not known to be Full.
(wat.core/defenum usr/Box :- [T] wat.enum/Pure
  :Full  [inside :- T]
  :Empty [])
(wat.core/defn user/takes-full [b :- (usr/Box.Full :- [wat.type/i64])] :- wat.type/nil
  (wat.kernel/println "ok"))
(wat.core/defn user/relay [b :- (usr/Box :- [wat.type/i64])] :- wat.type/nil
  (user/takes-full b))
(wat.core/defn user/main [] :- wat.type/nil (wat.kernel/println "ok"))
