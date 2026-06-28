;; tests/types/probe_arc293_ctor_parity_struct.wat — RED at HEAD: struct ctor parity not built

(:wat::core::defstruct :geo::SPt [x <- :wat::core::i64  y <- :wat::core::i64])
(:wat::core::defn :user::main [] -> :wat::core::i64
  (:geo::SPt/x (:geo::SPt 3 4)))
