;; tests/reflection/probe_arc241_stone7_metadata_of_reflection_c04.wat
;; Fixture for contract_04_def_without_metadata_returns_none.
;; def with NO metadata; metadata-of returns None.
(:wat::core::def :my::no-meta 42)
(:wat::core::defn :user::compute [] -> (:wat::core::Option :- [:wat::core::i64])
  (:wat::runtime::metadata-of :my::no-meta))
