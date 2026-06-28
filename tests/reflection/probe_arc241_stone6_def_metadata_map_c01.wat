;; tests/reflection/probe_arc241_stone6_def_metadata_map_c01.wat
;; Fixture for contract_01_def_with_doc_metadata_parses.
;; (def :name {:doc "..."} value) — single-entry metadata must parse cleanly.
(:wat::core::def :my::x
  {:doc "the x value"}
  42)
