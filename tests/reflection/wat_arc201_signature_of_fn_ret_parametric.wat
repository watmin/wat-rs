;; tests/reflection/wat_arc201_signature_of_fn_ret_parametric.wat
;; Fixture for test signature_of_fn_extracts_return_type_parametric.
;; Probe: parametric return type (Vector :- [i64]) lands as structured Bundle.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
              [f   (:wat::core::fn [] -> (:wat::core::Vector :- [:wat::core::i64])
                     (:wat::core::Vector :wat::core::i64))
               sig (:wat::runtime::signature-of-fn f)
               rendered sig]
              (:wat::kernel::println rendered)))
