;; tests/reflection/probe_arc241_stone6_def_metadata_map_c02.wat
;; Fixture for contract_02_def_with_multi_entry_metadata_parses.
;; (def :name {:k1 :v1 :k2 :v2} value) — multi-entry metadata must parse cleanly.
(:wat::core::def :my::y
  {:doc "documented"
   :deprecated true}
  100)
