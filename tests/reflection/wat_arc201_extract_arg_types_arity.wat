;; tests/reflection/wat_arc201_extract_arg_types_arity.wat
;; Fixture for test extract_arg_types_arity_matches_extract_arg_names.
;; Probe: 3-arg fn — extract-arg-types and extract-arg-names return same length.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
              [f     (:wat::core::fn [a <- :wat::core::i64 b <- :wat::core::String c <- :wat::core::i64]
                       -> :wat::core::String
                       b)
               sig   (:wat::runtime::signature-of-fn f)
               names (:wat::runtime::extract-arg-names sig)
               tys   (:wat::runtime::extract-arg-types sig)
               nlen  (:wat::core::length names)
               tlen  (:wat::core::length tys)]
              (:wat::kernel::println (:wat::edn::write nlen))
              (:wat::kernel::println (:wat::edn::write tlen))))
