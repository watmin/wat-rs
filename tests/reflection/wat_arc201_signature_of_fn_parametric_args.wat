;; tests/reflection/wat_arc201_signature_of_fn_parametric_args.wat
;; Fixture for test signature_of_fn_extracts_parametric_arg_types.
;; Probe: (Vector :- [i64]) param lands as structured Bundle (not flat keyword string).
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
              [f   (:wat::core::fn [xs <- (:wat::core::Vector :- [:wat::core::i64])] -> :wat::core::i64
                     42)
               sig (:wat::runtime::signature-of-fn f)
               rendered sig]
              (:wat::kernel::println rendered)))
