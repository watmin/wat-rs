;; tests/reflection/wat_arc201_signature_of_fn_ret_parametric.wat
;; Fixture for test signature_of_fn_extracts_return_type_parametric.
;; Probe: parametric return type (Vector :- [i64]) lands as structured Bundle.
(:wat::core::defn :user::main [] -> wat.type/nil
  (:wat::core::let
              [f   (:wat::core::fn [] -> (wat.type/Vector :- [wat.type/i64])
                     (wat.type/Vector :- [wat.type/i64]))
               sig (:wat::runtime::signature-of-fn f)
               rendered sig]
              (:wat::kernel::println rendered)))
