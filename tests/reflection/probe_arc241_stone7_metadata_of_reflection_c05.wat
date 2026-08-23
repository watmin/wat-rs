;; tests/reflection/probe_arc241_stone7_metadata_of_reflection_c05.wat
;; Fixture for contract_05_unknown_binding_returns_none.
;; Unknown name -> None (not an error).
(:wat::core::defn :user::compute [] -> (:wat::core::Option :- [:wat::core::i64])
  (:wat::runtime::metadata-of :my::nonexistent))
