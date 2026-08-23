;; tests/reflection/probe_arc241_stone7_metadata_of_reflection_c03.wat
;; Fixture for contract_03_multi_entry_metadata_returns_some.
;; Multi-entry metadata; metadata-of returns Some.
(:wat::core::def :my::y
  {:doc "documented"
   :deprecated true}
  100)
(:wat::core::defn :user::compute [] -> (:wat::core::Option :- [:wat::core::i64])
  (:wat::runtime::metadata-of :my::y))
