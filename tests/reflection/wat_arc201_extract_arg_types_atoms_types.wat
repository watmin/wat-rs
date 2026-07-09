;; tests/reflection/wat_arc201_extract_arg_types_atoms_types.wat
;; Fixture for test extract_arg_types_returns_atoms_for_monomorphic_args (part 1).
;; Probe: extract-arg-types on a 2-param fn prints the type Symbols.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
              [f    (:wat::core::fn [msg <- :wat::core::String count <- :wat::core::i64]
                      -> :wat::core::String
                      msg)
               sig  (:wat::runtime::signature-of-fn f)
               tys  (:wat::runtime::extract-arg-types sig)
               rendered tys]
              (:wat::kernel::println rendered)))
