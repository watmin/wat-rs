;; tests/reflection/probe_arc241_stone6_def_metadata_map_c05.wat
;; Fixture for contract_05_defn_without_metadata_unchanged.
;; Regression: defn without metadata must still work.
(:wat::core::defn :my::g
  [x <- :wat::core::i64] -> :wat::core::i64
  x)
