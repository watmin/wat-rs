;; tests/types/probe_arc237_s0_records_gate_typeunion.wat — T1b: macro-emitted typeunion synthesizes is-predicate

(wat.core/defmacro my/defnum
  [name :- wat.type/AST]
  :- wat.type/AST
  `(wat.core/typeunion ~name [wat.type/i64 wat.type/f64]))

(my/defnum my.g/Num)
