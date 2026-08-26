;; tests/reflection/probe_arc241_stone7_metadata_of_reflection_c02.wat
;; Fixture for contract_02_defn_with_metadata_returns_some.
;; defn with metadata; metadata-of on the binding name returns Some(map).
(:wat::core::defn :my::f
  {:doc "doubles x"}
  [x <- :wat::core::i64] -> :wat::core::i64
  (:wat::i64::+ x x))
(:wat::core::defn :user::compute [] -> (:wat::core::Option :- [:wat::core::i64])
  (:wat::runtime::metadata-of :my::f))
