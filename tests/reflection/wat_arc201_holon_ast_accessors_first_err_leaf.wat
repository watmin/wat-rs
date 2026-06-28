;; tests/reflection/wat_arc201_holon_ast_accessors_first_err_leaf.wat
;; Fixture for test bundle_first_errors_on_leaf_input.
;; Probe: Bundle/first on a leaf HolonAST must raise TypeMismatch.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
              [leaf (:wat::holon::leaf "hi")
               _    (:wat::holon::Bundle/first leaf)]
              (:wat::kernel::println "unreachable")))
