;; tests/reflection/wat_arc201_signature_of_fn_monomorphic_args.wat
;; Fixture for test signature_of_fn_extracts_monomorphic_arg_types.
;; Probe: Path-typed params land as atomic Symbols in the signature.
(:wat::core::defn :user::main [] -> wat.type/nil
  (:wat::core::let
              [f   (:wat::core::fn [n <- wat.type/i64 s <- wat.type/String] -> wat.type/String
                     s)
               sig (:wat::runtime::signature-of-fn f)
               rendered sig]
              (:wat::kernel::println rendered)))
