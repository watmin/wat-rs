;; tests/reflection/probe_diagnostic_typed_entities_reflection_p2.wat
;; Fixture for probe_2_extract_classifier_on_bare_atom.
;; extract-classifier on a non-canonical-wrap HolonAST returns None.
(:wat::core::defn :user::compute [] -> (:wat::core::Option :- [:wat::core::String])
  (:wat::core::let
      [bare (:wat::holon::Atom (:wat::holon::to-holon 42))]
      (:wat::holon::extract-classifier bare)))
