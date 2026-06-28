;; tests/types/probe_arc237_s0_records_gate_struct.wat — T1a: macro-emitted struct synthesizes is-predicate

(:wat::core::defmacro :my::defthing
  [name <- :wat::WatAST]
  -> :wat::WatAST
  `(:wat::core::defstruct ~name [n <- :wat::core::i64]))

(:my::defthing :my::g::Widget)
