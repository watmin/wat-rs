;; tests/reflection/wat_arc201_holon_ast_accessors_first_err_empty.wat
;; Fixture for test bundle_first_errors_on_empty_bundle.
;; Probe: Bundle/first on an empty Bundle must raise a runtime error.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
              [empty-res (:wat::holon::Bundle (:wat::core::Vector :- [:wat::holon::HolonAST]))
               empty     (:wat::core::match empty-res 
                           ((:wat::core::Ok b)  b)
                           ((:wat::core::Err _) (:wat::kernel::abort "empty Bundle construction failed")))
               _         (:wat::holon::Bundle/first empty)]
              (:wat::kernel::println "unreachable")))
