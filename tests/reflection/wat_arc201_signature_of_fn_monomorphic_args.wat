;; tests/reflection/wat_arc201_signature_of_fn_monomorphic_args.wat
;; Fixture for test signature_of_fn_extracts_monomorphic_arg_types.
;; Probe: Path-typed params land as atomic Symbols in the signature.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
              [f   (:wat::core::fn [n <- :wat::core::i64 s <- :wat::core::String] -> :wat::core::String
                     s)
               sig (:wat::runtime::signature-of-fn f)
               rendered sig]
              (:wat::kernel::println rendered)))
