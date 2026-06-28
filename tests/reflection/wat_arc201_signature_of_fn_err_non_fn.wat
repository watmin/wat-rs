;; tests/reflection/wat_arc201_signature_of_fn_err_non_fn.wat
;; Fixture for test signature_of_fn_errors_on_non_fn_input.
;; Probe: passing an i64 (not a fn) to signature-of-fn must raise TypeMismatch.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
              [_ (:wat::runtime::signature-of-fn 42)]
              (:wat::kernel::println "unreachable")))
