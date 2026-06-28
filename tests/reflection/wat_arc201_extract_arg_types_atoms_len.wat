;; tests/reflection/wat_arc201_extract_arg_types_atoms_len.wat
;; Fixture for test extract_arg_types_returns_atoms_for_monomorphic_args (part 2).
;; Probe: extract-arg-types on the same 2-param fn prints the length (must be 2).
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
              [f    (:wat::core::fn [msg <- :wat::core::String count <- :wat::core::i64]
                      -> :wat::core::String
                      msg)
               sig  (:wat::runtime::signature-of-fn f)
               tys  (:wat::runtime::extract-arg-types sig)
               len  (:wat::core::length tys)]
              (:wat::kernel::println (:wat::edn::write len))))
