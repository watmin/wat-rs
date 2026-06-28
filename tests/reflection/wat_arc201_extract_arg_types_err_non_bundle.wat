;; tests/reflection/wat_arc201_extract_arg_types_err_non_bundle.wat
;; Fixture for test extract_arg_types_errors_on_non_bundle_input.
;; Probe: passing an i64 (not HolonAST) to extract-arg-types must raise TypeMismatch.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
              [_ (:wat::runtime::extract-arg-types 42)]
              (:wat::kernel::println "unreachable")))
