;; tests/reflection/wat_arc201_extract_arg_types_bundles.wat
;; Fixture for test extract_arg_types_returns_bundles_for_parametric_args.
;; Probe: (Vector :- [i64]) param type lands as a structured Bundle, not a flat keyword.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
              [f    (:wat::core::fn [xs <- (:wat::core::Vector :- [:wat::core::i64])]
                      -> :wat::core::i64
                      42)
               sig  (:wat::runtime::signature-of-fn f)
               tys  (:wat::runtime::extract-arg-types sig)
               rendered tys]
              (:wat::kernel::println rendered)))
