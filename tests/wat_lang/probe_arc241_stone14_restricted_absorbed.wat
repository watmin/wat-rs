;; tests/wat_lang/probe_arc241_stone14_restricted_absorbed.wat
;; C01: :restricted-to metadata-map — allowed caller (same namespace) passes.

(:wat::core::defn :test::restricted-target
  {:restricted-to [:test::]}
  [] -> :wat::core::i64 42)
(:wat::core::defn :test::allowed-caller
  [] -> :wat::core::i64 (:test::restricted-target))
