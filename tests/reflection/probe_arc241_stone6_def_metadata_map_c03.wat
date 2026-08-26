;; tests/reflection/probe_arc241_stone6_def_metadata_map_c03.wat
;; Fixture for contract_03_defn_with_metadata_inherits_via_macro.
;; defn-with-metadata expands to (def :name {meta} (fn ...)) — must parse cleanly.
(:wat::core::defn :my::f
  {:doc "doubles its input"}
  [x <- :wat::core::i64] -> :wat::core::i64
  (:wat::i64::+ x x))
