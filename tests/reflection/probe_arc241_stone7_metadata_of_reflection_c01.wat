;; tests/reflection/probe_arc241_stone7_metadata_of_reflection_c01.wat
;; Fixture for contract_01_def_with_metadata_returns_some.
;; def with single-entry metadata; metadata-of returns Some(map).
(:wat::core::def :my::x
  {:doc "the x value"}
  42)
(:wat::core::defn :user::compute [] -> (:wat::core::Option :- [:wat::core::i64])
  (:wat::runtime::metadata-of :my::x))
