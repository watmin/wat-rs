;; tests/types/probe_arc237_s0_records_gate_typeunion.wat — T1b: macro-emitted typeunion synthesizes is-predicate

(:wat::core::defmacro :my::defnum
  [name <- :wat::WatAST]
  -> :wat::WatAST
  `(:wat::core::typeunion ~name [:wat::core::i64 :wat::core::f64]))

(:my::defnum :my::g::Num)
