;; tests/reflection/probe_diagnostic_typed_entities_reflection_p7.wat
;; Fixture for probe_7_bind_left_on_non_bind.
;; Bind/left on a non-Bind HolonAST returns None.
(:wat::core::defn :user::compute [] -> (:wat::core::Option :- [:wat::holon::HolonAST])
  (:wat::core::let
      [bare (:wat::holon::Atom (:wat::holon::to-holon 42))]
      (:wat::holon::Bind/left bare)))
