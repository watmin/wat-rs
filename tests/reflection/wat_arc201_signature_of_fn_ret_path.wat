;; tests/reflection/wat_arc201_signature_of_fn_ret_path.wat
;; Fixture for test signature_of_fn_extracts_return_type_path.
;; Probe: atomic return type :wat::core::i64 appears at tail of signature.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
              [f   (:wat::core::fn [] -> :wat::core::i64 7)
               sig (:wat::runtime::signature-of-fn f)
               rendered (:wat::edn::write sig)]
              (:wat::kernel::println rendered)))
