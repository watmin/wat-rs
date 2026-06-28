;; tests/reflection/wat_arc201_holon_ast_accessors_children_err_atom.wat
;; Fixture for test bundle_children_errors_on_atom_input.
;; Probe: Bundle/children on a leaf HolonAST must raise TypeMismatch.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
              [leaf (:wat::holon::leaf 42)
               _    (:wat::holon::Bundle/children leaf)]
              (:wat::kernel::println "unreachable")))
