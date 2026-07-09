;; tests/reflection/wat_arc201_signature_of_fn_compose_names.wat
;; Fixture for test signature_of_fn_composes_with_extract_arg_names.
;; Probe: signature-of-fn output composes with extract-arg-names.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
              [f      (:wat::core::fn [logger <- :wat::core::String counter <- :wat::core::i64]
                       -> :wat::core::String
                       logger)
               sig    (:wat::runtime::signature-of-fn f)
               names  (:wat::runtime::extract-arg-names sig)
               rendered names]
              (:wat::kernel::println rendered)))
