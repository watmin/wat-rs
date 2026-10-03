;; Stone 255.25a — a variant is a type the moment its enum is (arc 255).
;; ROW — a Pure enum's variant in an extend-type TARGET: checks, runs, prints. Pre-stone: EdgeFreeTypeName.
(wat.core/defenum probe/Tr wat.enum/Pure
  :Sh []
  :Wi [])
(wat.core/defsurface probe/Greets :- [T] :nature wat.type/Struct
  :features [(greet [self :- (probe/Greets :- [T])] :- wat.type/String)])
(wat.core/defstruct probe/Box [])
(wat.core/extend-type probe/Box (probe/Greets :- [probe/Tr.Wi])
  (greet [self] :- wat.type/String "hello"))
(wat.core/defn user/main [] :- wat.type/nil
  (wat.kernel/println (probe.Greets/greet (probe/Box))))
