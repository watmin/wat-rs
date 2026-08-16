;; tests/types/probe_arc293_ctor_parity_newtype.wat — RED at HEAD: newtype ctor parity not built

(:wat::core::newtype :my::Amount :wat::core::i64)
(:wat::core::defn :probe::drive [] -> :my::Amount
  (:my::Amount 42))
