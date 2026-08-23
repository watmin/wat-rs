;; tests/reflection/probe_diagnostic_typed_entities_reflection_p4.wat
;; Fixture for probe_4_bind_right_on_non_bind.
;; Bind/right on a non-Bind HolonAST returns None.
(:wat::core::defn :user::compute [] -> (:wat::core::Option :- [:wat::holon::HolonAST])
  (:wat::core::let
      [bare (:wat::holon::Atom (:wat::holon::to-holon 42))]
      (:wat::holon::Bind/right bare)))
